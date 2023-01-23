import { defineStore } from 'pinia'
import { Ref, ref, watch, computed } from 'vue'
import { getProjectFolder, cacheProjectFolder } from '@/fs'
import { Engine } from '@/engine/engine'
import { Runtime } from '@/Runtime'

export const useWorkspaceStore = defineStore('WorkspaceStore', () => {
  let projectFolder: Ref<FileSystemDirectoryHandle | undefined> = ref(undefined)
  let engine: Ref<Engine | undefined> = ref(undefined)
  let frame: Ref<number> = ref(0)
  let length: Ref<number> = ref(60)
  let reloadCount: Ref<number> = ref(0)
  let data: Ref<any> = ref({
    project: {
      markers: [],
    },
    editor: {
      volume: 1,
    },
  })
  let error: Ref<string | null> = ref(null)
  const audioContext: Ref<AudioContext> = ref(
    new AudioContext({
      latencyHint: 'interactive',
    })
  )
  const audioGain: Ref<GainNode> = ref(audioContext.value.createGain())
  const audioDestination: Ref<AudioNode> = ref(audioContext.value.createGain())
  const setupAudio: Ref<boolean> = ref(false)

  if (!setupAudio.value) {
    setupAudio.value

    audioGain.value.connect(audioContext.value.destination)

    audioDestination.value = audioGain.value
  }

  async function loadProject(name: string) {
    projectFolder.value = (await getProjectFolder(name)) || undefined

    if (!projectFolder.value) return

    cacheProjectFolder(projectFolder.value)

    const runtime = new Runtime(projectFolder.value)

    engine.value = new Engine(runtime)
    await engine.value.load()

    length.value = engine.value.length
    frame.value = 0

    error.value = null

    try {
      const loadedData = JSON.parse(
        await (
          await (await projectFolder.value.getFileHandle('data.json')).getFile()
        ).text()
      )

      if (
        typeof loadedData.project != 'object' ||
        Array.isArray(loadedData.project) ||
        loadedData.project == null
      )
        throw new Error()

      if (!Array.isArray(loadedData.project.markers)) throw new Error()

      for (const marker of loadedData.project.markers) {
        if (
          typeof marker != 'object' ||
          Array.isArray(marker) ||
          marker == null
        )
          throw new Error()

        if (typeof marker.name != 'string') throw new Error()

        if (typeof marker.id != 'string') throw new Error()

        if (typeof marker.frame != 'number') throw new Error()
      }

      if (
        typeof loadedData.editor != 'object' ||
        Array.isArray(loadedData.editor) ||
        loadedData.editor == null
      )
        if (typeof loadedData.editor.volume != 'number') throw new Error()

      data.value = loadedData
    } catch {
      error.value =
        'There was an error loading your project! Make sure you have a data.json file and that it is formated correctly.'
    }

    reloadCount.value++
  }

  async function loadProjectFromCache() {
    await loadProject('')
  }

  async function render() {
    return await engine.value?.render()
  }

  async function updateFrame(frameNumber: number) {
    if (!engine.value) return

    if (frameNumber == frame.value) return

    if (frameNumber < frame.value) {
      await engine.value.reloadContext()

      for (let frameOffset = 0; frameOffset <= frameNumber; frameOffset++) {
        await engine.value.next()
      }

      frame.value = frameNumber

      return
    }

    for (
      let frameOffset = frame.value;
      frameOffset < frameNumber;
      frameOffset++
    ) {
      await engine.value.next()
    }

    frame.value = frameNumber
  }

  function generateUuid() {
    let d = new Date().getTime()
    let d2 =
      (typeof performance !== 'undefined' &&
        performance.now &&
        performance.now() * 1000) ||
      0
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(
      /[xy]/g,
      function (c) {
        let r = Math.random() * 16
        if (d > 0) {
          r = (d + r) % 16 | 0
          d = Math.floor(d / 16)
        } else {
          r = (d2 + r) % 16 | 0
          d2 = Math.floor(d2 / 16)
        }
        return (c === 'x' ? r : (r & 0x3) | 0x8).toString(16)
      }
    )
  }

  async function createMarker(name: string, frame: number) {
    if (!projectFolder.value) return

    data.value.project.markers.push({
      name,
      frame,
      id: generateUuid(),
    })

    const dataFile = await projectFolder.value.getFileHandle('data.json')

    // @ts-ignore
    const writable = await dataFile.createWritable()
    await writable.write(JSON.stringify(data.value, null, 2))
    await writable.close()
  }

  async function updateMarker(id: string, name: string, frame: number) {
    if (!projectFolder.value) return

    const index = data.value.project.markers.findIndex(
      (marker: any) => marker.id == id
    )

    if (index == -1) return

    data.value.project.markers[index] = {
      name,
      frame,
      id,
    }

    const dataFile = await projectFolder.value.getFileHandle('data.json')

    // @ts-ignore
    const writable = await dataFile.createWritable()
    await writable.write(JSON.stringify(data.value, null, 2))
    await writable.close()
  }

  function getAudioBuffer() {
    if (!engine.value) return null

    if (!engine.value.audioBuffer) return null

    return engine.value.audioBuffer
  }

  const frameRate = computed(() => engine.value?.frameRate || 60)

  const markers = computed(() => {
    let markers = []

    for (const marker of data.value.project.markers) {
      markers.push(marker)
    }

    return markers.sort((markerA, markerB) => markerA.frame - markerB.frame)
  })

  const volumePerFrame = computed(() => {
    if (!engine.value) return []

    return engine.value.volumePerFrame
  })

  return {
    loadProject,
    loadProjectFromCache,
    render,
    frame,
    length,
    reloadCount,
    updateFrame,
    frameRate,
    projectFolder,
    error,
    createMarker,
    markers,
    updateMarker,
    volumePerFrame,
    getAudioBuffer,
    audioContext,
    audioGain,
    audioDestination,
  }
})

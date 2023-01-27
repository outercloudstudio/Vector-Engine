import { defineStore } from 'pinia'
import { Ref, ref, computed, watch } from 'vue'
import { getProjectFolder, cacheProjectFolder } from '@/fs'
import { Engine } from '@/engine/engine'
import { Runtime } from '@/Runtime'

export const useWorkspaceStore = defineStore('WorkspaceStore', () => {
  let projectFolder: Ref<FileSystemDirectoryHandle | undefined> = ref(undefined)
  let runtime: any = ref(new Runtime(null))
  let engine: Ref<Engine | undefined> = ref(undefined)
  let data: Ref<any> = ref({
    project: {
      markers: [],
    },
    editor: {
      volume: 1,
    },
  })
  let error: Ref<string | null> = ref(null)
  let loaded: Ref<boolean> = ref(false)

  let frame: Ref<number> = ref(0)
  let length: Ref<number> = ref(60)

  const audioContext: Ref<AudioContext> = ref(
    new AudioContext({
      latencyHint: 'interactive',
    })
  )
  const audioGain: Ref<GainNode> = ref(audioContext.value.createGain())
  const audioDestination: Ref<AudioNode> = ref(audioContext.value.createGain())
  const setupAudio: Ref<boolean> = ref(false)

  const sceneInference: Ref<{ name: string; frame: number }[]> = ref([])

  if (!setupAudio.value) {
    setupAudio.value

    audioGain.value.connect(audioContext.value.destination)

    audioDestination.value = audioGain.value
  }

  let audioInferenceVolumeCache: Ref<number[]> = ref([])

  async function loadData() {
    if (!projectFolder.value) return

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
        throw new Error()

      if (typeof loadedData.editor.volume != 'number') throw new Error()

      if (
        typeof loadedData.editor.inference != 'object' ||
        Array.isArray(loadedData.editor.inference) ||
        loadedData.editor.inference == null
      )
        throw new Error()

      if (typeof loadedData.editor.inference.audio != 'boolean')
        throw new Error()
      if (typeof loadedData.editor.inference.scenes != 'boolean')
        throw new Error()

      data.value = loadedData
    } catch {
      error.value =
        'There was an error loading your project! Make sure you have a data.json file and that it is formated correctly.'
    }
  }

  async function runInferences(engine: Engine) {
    sceneInference.value = []

    if (!inferenceScenes.value) return

    let inference: { name: string; frame: number }[] = []

    for (let frame = 0; frame < length.value; frame++) {
      const scene = engine.activeScenes[engine.activeScenes.length - 1]

      if (scene) {
        const sceneName = Object.values(engine.scenes).includes(scene.path)
          ? Object.keys(engine.scenes)[
              Object.values(engine.scenes).findIndex(path => path == scene.path)
            ]
          : scene.path

        if (
          inference[inference.length - 1] == undefined ||
          inference[inference.length - 1].name != sceneName
        ) {
          inference.push({
            name: sceneName,
            frame,
          })
        }
      }

      await engine.next()
    }

    sceneInference.value = inference
  }

  async function loadProject(name: string) {
    projectFolder.value = (await getProjectFolder(name)) || undefined

    if (!projectFolder.value) return

    cacheProjectFolder(projectFolder.value)

    await loadData()

    runtime.value.reload(projectFolder.value)

    engine.value = new Engine(
      runtime.value,
      data.value.project.markers,
      inferenceAudio.value
    )
    await engine.value.load()

    length.value = engine.value.length
    frame.value = 0

    loadData()

    const inferenceEngine = new Engine(
      runtime.value,
      data.value.project.markers,
      false
    )
    await inferenceEngine.load()

    runInferences(inferenceEngine)

    loaded.value = true
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

      for (let frameOffset = 0; frameOffset < frameNumber; frameOffset++) {
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

    if (
      engine.value.audioBuffer == null ||
      engine.value.volumePerFrame.length == 0
    )
      return audioInferenceVolumeCache.value

    return engine.value.volumePerFrame
  })

  watch(volumePerFrame, volumePerFrame => {
    if (!engine.value) return

    if (engine.value.audioBuffer == null) return

    audioInferenceVolumeCache.value = volumePerFrame
  })

  const volume = computed(() => {
    return data.value.editor.volume
  })

  async function updateVolume(volume: number) {
    if (!projectFolder.value) return

    data.value.editor.volume = volume

    const dataFile = await projectFolder.value.getFileHandle('data.json')

    // @ts-ignore
    const writable = await dataFile.createWritable()
    await writable.write(JSON.stringify(data.value, null, 2))
    await writable.close()
  }

  const inferenceAudio = computed(() => {
    return data.value.editor.inference.audio
  })

  async function updateInferenceAudio(inference: boolean) {
    if (!projectFolder.value) return

    data.value.editor.inference.audio = inference

    const dataFile = await projectFolder.value.getFileHandle('data.json')

    // @ts-ignore
    const writable = await dataFile.createWritable()
    await writable.write(JSON.stringify(data.value, null, 2))
    await writable.close()
  }

  const inferenceScenes = computed(() => {
    return data.value.editor.inference.scenes
  })

  async function updateInferenceScenes(inference: boolean) {
    if (!projectFolder.value) return

    data.value.editor.inference.scenes = inference

    const dataFile = await projectFolder.value.getFileHandle('data.json')

    // @ts-ignore
    const writable = await dataFile.createWritable()
    await writable.write(JSON.stringify(data.value, null, 2))
    await writable.close()
  }

  return {
    loadProject,
    loadProjectFromCache,
    render,
    frame,
    length,
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
    loaded,
    sceneInference,
    volume,
    updateVolume,
    inferenceAudio,
    updateInferenceAudio,
    inferenceScenes,
    updateInferenceScenes,
  }
})

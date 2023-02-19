import { defineStore } from 'pinia'
import { Ref, ref, computed, watch } from 'vue'
import { Engine } from '@/engine/Engine'

export const useEngineStore = defineStore('EngineStore', () => {
  const project: Ref<any> = ref(undefined)
  const engine: Ref<Engine | undefined> = ref(undefined)
  const reloadEngineEvent: Ref<number> = ref(0)
  const data: Ref<any> = ref(undefined)
  const updatedDataEvent: Ref<number> = ref(0)
  const blockingErrors: Ref<string[]> = ref([])
  const errors: Ref<string[]> = ref([])
  const frame: Ref<number> = ref(0)

  async function makeEngine(newProject: any, newData: any) {
    project.value = newProject
    data.value = newData

    engine.value = new Engine(
      newProject,
      newData.project.markers,
      false,
      (error: any) => {
        errors.value.push(error)
      }
    )

    await engine.value.load()

    reloadEngineEvent.value++
  }

  async function remakeEngine(newProject: any) {
    project.value = newProject

    engine.value = new Engine(
      newProject,
      data.value.project.markers,
      false,
      (error: any) => {
        errors.value.push(error)
      }
    )

    await engine.value.load()

    console.warn('Loaded engine...')

    await setFrame(frame.value)

    reloadEngineEvent.value++
  }

  async function render(frame: number): Promise<OffscreenCanvas> {
    if (!engine.value)
      throw new Error('Tried rendering before engine was made!')

    if (!engine.value.loaded)
      throw new Error('Tried rendering before engine was loaded!')

    return await engine.value.render()
  }

  async function setFrame(newFrame: number) {
    if (!loaded) return
    if (!engine.value) return

    if (engine.value.frame > newFrame) await engine.value.reload()

    for (
      let engineFrame = engine.value.frame;
      engineFrame < newFrame;
      engineFrame++
    ) {
      await engine.value.next()
    }

    frame.value = newFrame
  }

  const loaded = computed(() => engine.value && engine.value.loaded)
  const frameRate = computed(() =>
    loaded.value ? engine.value!.frameRate : 60
  )
  const length = computed(() => (loaded.value ? engine.value!.length : 60))
  const markers = computed(() =>
    loaded.value && blockingErrors.value.length == 0
      ? engine.value!.markers
      : []
  )
  const audioTrack = computed(() =>
    loaded.value && blockingErrors.value.length == 0
      ? engine.value!.audioTrack
      : undefined
  )

  watch(updatedDataEvent, async () => {
    if (!engine.value) return
    if (!loaded.value) return

    await makeEngine(project.value, data.value)

    await setFrame(frame.value)

    reloadEngineEvent.value++
  })

  return {
    project,
    makeEngine,
    remakeEngine,
    loaded,
    reloadEngineEvent,
    render,
    frame,
    setFrame,
    frameRate,
    length,
    blockingErrors,
    errors,
    markers,
    data,
    updatedDataEvent,
    audioTrack,
  }
})

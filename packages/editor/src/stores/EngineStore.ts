import { defineStore } from 'pinia'
import { Ref, ref, computed, watch } from 'vue'
import { Engine } from '@/engine/Engine'

export const useEngineStore = defineStore('EngineStore', () => {
  const engine: Ref<Engine | undefined> = ref(undefined)
  const blockingErrors: Ref<string[]> = ref([])
  const frame: Ref<number> = ref(0)

  async function makeEngine(project: any) {
    engine.value = new Engine(project, [], false)
    await engine.value.load()
  }

  async function render(frame: number): Promise<OffscreenCanvas> {
    if (!engine.value)
      throw new Error('Tried rendering before engine was made!')

    if (!engine.value.loaded)
      throw new Error('Tried rendering before engine was loaded!')

    return await engine.value.render()
  }

  async function setFrame(newFrame: number) {
    frame.value = newFrame

    if (!loaded) return
    if (!engine.value) return

    if (engine.value.frame < newFrame) {
      for (
        let engineFrame = engine.value.frame;
        engineFrame < newFrame;
        engineFrame++
      ) {
        await engine.value.next()
      }
    }

    console.log(frame.value)
  }

  const loaded = computed(() => engine.value && engine.value.loaded)
  const frameRate = computed(() => (loaded ? engine.value!.frameRate : 60))
  const length = computed(() => (loaded ? engine.value!.length : 60))

  return {
    makeEngine,
    loaded,
    render,
    frame,
    setFrame,
    frameRate,
    length,
    blockingErrors,
  }
})

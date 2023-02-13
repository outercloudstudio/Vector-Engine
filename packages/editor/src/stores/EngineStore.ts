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

  const loaded = computed(() => engine.value && engine.value.loaded)

  return {
    makeEngine,
    render,
    frame,
    blockingErrors,
    loaded,
  }
})

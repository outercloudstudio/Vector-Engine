import { defineStore } from 'pinia'
import { Ref, ref, computed } from 'vue'
import { useEngineStore } from '@/stores/EngineStore'

export const useEditorStore = defineStore('EditorStore', () => {
  const EngineStore = useEngineStore()

  const playing: Ref<boolean> = ref(false)
  const speed: Ref<number> = ref(1)
  const startedPlayingTime: Ref<number> = ref(0)
  const startedPlayingFrame: Ref<number> = ref(0)
  const looping: Ref<boolean> = ref(false)
  const loopingStart = ref(0)
  const loopingEnd = ref(0)

  const selectedMarker: Ref<string> = ref('')

  function play() {
    playing.value = true
    startedPlayingTime.value = Date.now()

    if (looping.value) {
      // if (
      //   EngineStore.frame < loopingStart.value &&
      //   loopingStart.value < WorkspaceStore.length
      // ) {
      //   await WorkspaceStore.updateFrame(loopingStart.value)
      // }
      // if (WorkspaceStore.frame > loopingEnd.value && loopingEnd.value >= 0) {
      //   await WorkspaceStore.updateFrame(loopingEnd.value)
      // }
    }

    startedPlayingFrame.value = EngineStore.frame

    requestAnimationFrame(playUpdate)
  }

  async function playUpdate() {
    if (!playing.value) return

    const now = Date.now()
    const newFrame =
      Math.floor(
        ((now - startedPlayingTime.value) / 1000) *
          speed.value *
          EngineStore.frameRate
      ) + startedPlayingFrame.value

    await EngineStore.setFrame(newFrame)

    if (
      EngineStore.frame >= EngineStore.length - 1 ||
      (looping.value &&
        EngineStore.frame >= loopingEnd.value &&
        loopingEnd.value >= 0)
    ) {
      startedPlayingTime.value = Date.now()

      startedPlayingFrame.value =
        looping.value && loopingStart.value < length ? loopingStart.value : 0

      // stopAudioPlayback()
      // await startAudioPlayback(startedFrame / WorkspaceStore.frameRate)
    }

    requestAnimationFrame(playUpdate)
  }

  function pause() {
    playing.value = false

    // stopAudioPlayback()
  }

  function restart() {
    pause()

    EngineStore.setFrame(0)
  }

  function next() {
    pause()

    if (EngineStore.frame >= EngineStore.length - 1) return

    EngineStore.setFrame(EngineStore.frame + 1)
  }

  function back() {
    pause()

    if (EngineStore.frame <= 0) return

    EngineStore.setFrame(EngineStore.frame - 1)
  }

  return {
    playing,
    play,
    pause,
    restart,
    next,
    back,
    speed,
    selectedMarker,
  }
})

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

  const muted: Ref<boolean> = ref(false)

  async function play() {
    playing.value = true
    startedPlayingTime.value = Date.now()

    if (looping.value) {
      if (
        EngineStore.frame < loopingStart.value &&
        loopingStart.value < EngineStore.length
      ) {
        await EngineStore.setFrame(loopingStart.value)
      }
      if (EngineStore.frame > loopingEnd.value && loopingEnd.value >= 0) {
        await EngineStore.setFrame(loopingEnd.value)
      }
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
        looping.value &&
        loopingStart.value < EngineStore.length &&
        loopingStart.value >= 0
          ? loopingStart.value
          : 0

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

  function uuid() {
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

  function createMarker(name: string, frame: number) {
    if (!EngineStore.loaded) return
    if (EngineStore.blockingErrors.length > 0) return

    EngineStore.data.project.markers.push({
      name,
      frame,
      id: uuid(),
    })

    EngineStore.updatedDataEvent++
  }

  function deleteMarker(id: string) {
    EngineStore.data.project.markers.splice(
      EngineStore.data.project.markers.findIndex(
        (marker: any) => marker.id == id
      ),
      1
    )

    EngineStore.updatedDataEvent++

    if (import.meta.hot) {
      import.meta.hot.send('vector-engine:data_update', EngineStore.data)
    }
  }

  function updateMarker(id: string, name: string, frame: number) {
    const index = EngineStore.data.project.markers.findIndex(
      (marker: any) => marker.id == id
    )

    if (index == -1) return

    EngineStore.data.project.markers[index] = {
      name,
      frame,
      id,
    }

    EngineStore.updatedDataEvent++
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
    muted,
    looping,
    loopingStart,
    loopingEnd,
    createMarker,
    deleteMarker,
    updateMarker,
  }
})

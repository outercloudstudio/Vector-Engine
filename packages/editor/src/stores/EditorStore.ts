import { defineStore } from 'pinia'
import { computed, Ref, ref, watch } from 'vue'
import { useEngineStore } from '../stores/EngineStore'
import { Engine } from '@vector-engine/core'

export const useEditorStore = defineStore('EditorStore', () => {
  const EngineStore = useEngineStore()

  const audioInference: Ref<number[]> = ref([])

  const playing: Ref<boolean> = ref(false)
  const speed: Ref<number> = ref(1)
  const startedPlayingTime: Ref<number> = ref(0)
  const startedPlayingFrame: Ref<number> = ref(0)
  const looping: Ref<boolean> = ref(false)
  const loopingStart = ref(0)
  const loopingEnd = ref(0)

  const selectedMarker: Ref<string | undefined> = ref(undefined)

  const muted: Ref<boolean> = ref(false)

  const exportInProgress = ref(false)
  const exportProgress = ref(0)

  const audioContext: Ref<AudioContext> = ref(
    new AudioContext({
      latencyHint: 'interactive',
    })
  )
  const audioGain: Ref<GainNode> = ref(audioContext.value.createGain())
  const audioDestination: Ref<AudioNode> = ref(audioContext.value.createGain())
  const audioTrackBufferSource: Ref<AudioBufferSourceNode | undefined> =
    ref(undefined)
  const setupAudio: Ref<boolean> = ref(false)

  if (!setupAudio.value) {
    setupAudio.value = true

    audioGain.value.connect(audioContext.value.destination)

    audioDestination.value = audioGain.value
  }

  const inferenceAudio = computed(() =>
    EngineStore.loaded ? EngineStore.data.editor.inference.audio : false
  )

  async function updateInferenceAudio(inference: boolean) {
    if (!EngineStore.loaded) return
    if (EngineStore.blockingErrors.length > 0) return

    EngineStore.data.editor.inference.audio = inference

    window.dispatchEvent(
      new CustomEvent('update-data', { detail: EngineStore.data })
    )

    EngineStore.updatedDataEvent++
  }

  const inferenceScenes = computed(() =>
    EngineStore.loaded ? EngineStore.data.editor.inference.scenes : false
  )

  async function updateInferenceScenes(inference: boolean) {
    if (!EngineStore.loaded) return
    if (EngineStore.blockingErrors.length > 0) return

    EngineStore.data.editor.inference.scenes = inference

    window.dispatchEvent(
      new CustomEvent('update-data', { detail: EngineStore.data })
    )

    EngineStore.updatedDataEvent++
  }

  watch(() => EngineStore.reloadEngineEvent, runInferences)

  async function runAudioInferences() {
    if (!inferenceAudio.value) {
      audioInference.value = []

      return
    }

    if (!EngineStore.audioTrack) return

    const channels = []

    for (
      let channelIndex = 0;
      channelIndex < EngineStore.audioTrack.numberOfChannels;
      channelIndex++
    ) {
      channels.push(EngineStore.audioTrack.getChannelData(channelIndex))
    }

    let volumePerFrame: number[] = []

    const samplesPerFrame =
      (EngineStore.audioTrack.sampleRate || 0) / EngineStore.frameRate

    let peak = 0

    for (let frame = 0; frame < EngineStore.length; frame++) {
      const startingSample = Math.floor(samplesPerFrame * frame)
      let frameValue = 0

      for (const channel of channels) {
        for (
          let sample = startingSample;
          sample <
          Math.min(
            startingSample + Math.floor(samplesPerFrame),
            channel.length
          );
          sample++
        ) {
          frameValue = Math.max(Math.abs(channel[sample]), frameValue)
        }
      }

      const squaredValue = Math.pow(frameValue, 3)

      volumePerFrame.push(squaredValue)

      peak = Math.max(squaredValue, peak)
    }

    for (let frame = 0; frame < EngineStore.length; frame++) {
      volumePerFrame[frame] = Math.min(
        volumePerFrame[frame] / Math.max(peak, 0.0001),
        1
      )
    }

    audioInference.value = volumePerFrame
  }

  async function runInferences() {
    await runAudioInferences()
  }

  async function startAudioPlayback(time: number) {
    const audioBuffer = EngineStore.audioTrack

    if (!audioBuffer) return

    audioTrackBufferSource.value = audioContext.value.createBufferSource()
    audioTrackBufferSource.value.buffer = audioBuffer
    audioTrackBufferSource.value.connect(audioDestination.value)

    audioContext.value.resume()

    audioTrackBufferSource.value.start(0, time)

    // while (audioContext.value.state == 'suspended') {
    //   await new Promise<void>(res => {
    //     setTimeout(() => {
    //       res()
    //     }, 1)
    //   })
    // }
  }

  function stopAudioPlayback() {
    if (!audioTrackBufferSource.value) return

    audioTrackBufferSource.value.stop()
  }

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

    await startAudioPlayback(startedPlayingFrame.value / EngineStore.frameRate)

    requestAnimationFrame(playUpdate)
  }

  async function playUpdate() {
    if (!playing.value) return

    const now = Date.now()
    let newFrame =
      Math.floor(
        ((now - startedPlayingTime.value) / 1000) *
          speed.value *
          EngineStore.frameRate
      ) + startedPlayingFrame.value

    if (newFrame > EngineStore.length - 1) newFrame = EngineStore.length - 1

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

      stopAudioPlayback()

      await startAudioPlayback(
        startedPlayingFrame.value / EngineStore.frameRate
      )
    }

    requestAnimationFrame(playUpdate)
  }

  function pause() {
    playing.value = false

    stopAudioPlayback()
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

    window.dispatchEvent(
      new CustomEvent('update-data', { detail: EngineStore.data })
    )

    EngineStore.updatedDataEvent++
  }

  function deleteMarker(id: string) {
    EngineStore.data.project.markers.splice(
      EngineStore.data.project.markers.findIndex(
        (marker: any) => marker.id == id
      ),
      1
    )

    window.dispatchEvent(
      new CustomEvent('update-data', { detail: EngineStore.data })
    )

    EngineStore.updatedDataEvent++
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

    window.dispatchEvent(
      new CustomEvent('update-data', { detail: EngineStore.data })
    )

    EngineStore.updatedDataEvent++
  }

  const volume = computed(() =>
    EngineStore.loaded ? EngineStore.data.editor.volume : 0
  )

  async function updateVolume(volume: number) {
    EngineStore.data.editor.volume = volume

    if (!muted.value) audioGain.value.gain.value = volume

    window.dispatchEvent(
      new CustomEvent('update-data', { detail: EngineStore.data })
    )

    EngineStore.updatedDataEvent++
  }

  async function exportAnimation(name: string) {
    if (exportProgress.value) return

    exportInProgress.value = true
    exportProgress.value = 0

    window.dispatchEvent(
      new CustomEvent('export-start', {
        detail: {
          name: name,
        },
      })
    )

    const engine = new Engine(
      EngineStore.project,
      EngineStore.scenes,
      EngineStore.markers,
      false
    )
    await engine.load()

    const frameDigits = engine.length.toString().length

    for (let frame = 0; frame < engine.length; frame++) {
      const frameName = `${name}/frame_${frame
        .toString()
        .padStart(frameDigits, '0')}.png`

      await engine.next()

      const render = await engine.render()
      const renderBlob: Blob = await (<any>render).convertToBlob()
      const arrayBuffer = await renderBlob.arrayBuffer()
      const byteArray = new Uint8Array(arrayBuffer, 0, arrayBuffer.byteLength)

      window.dispatchEvent(
        new CustomEvent('export', {
          detail: {
            name: frameName,
            image: byteArray,
          },
        })
      )

      exportProgress.value = (frame / engine.length) * 100
    }

    window.dispatchEvent(
      new CustomEvent('export-complete', {
        detail: {
          name,
          length: EngineStore.length,
          frameRate: EngineStore.frameRate,
        },
      })
    )

    exportInProgress.value = false
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
    audioInference,
    inferenceAudio,
    updateInferenceAudio,
    inferenceScenes,
    updateInferenceScenes,
    audioContext,
    audioGain,
    audioDestination,
    updateVolume,
    volume,
    exportAnimation,
    exportInProgress,
    exportProgress,
  }
})

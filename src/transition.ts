import router from '@/router'
import { ref } from 'vue'

export function useTransition(pageName: string, length: number) {
  // router.push({ name })

  return {
    transitioning: ref(false),
    progress: ref(0),
    transition() {
      this.transitioning.value = true

      const me = this

      let lastFrameTime = -1

      function frame(timestamp: DOMHighResTimeStamp) {
        const deltaTime = (timestamp - lastFrameTime) / 1000

        if (lastFrameTime == -1) {
          lastFrameTime = timestamp

          window.requestAnimationFrame(frame)

          return
        }

        lastFrameTime = timestamp

        me.progress.value += deltaTime

        if (me.progress.value > 1) me.progress.value = 1

        if (me.progress.value < 1) window.requestAnimationFrame(frame)
      }

      window.requestAnimationFrame(frame)
    },
  }
}

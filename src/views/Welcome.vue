<template>
  <NavBarVue leftIcon="dataset" leftLink="Projects" />

  <div id="page">
    <canvas ref="canvas"></canvas>

    <div id="wrapper">
      <h1 :class="{ transitioning: projectsTransition.transitioning.value }">
        Vector Engine
      </h1>

      <button
        :class="{ transitioning: projectsTransition.transitioning.value }"
        @click="projectsTransition.transition"
      >
        Let's Go!
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, Ref } from 'vue'
import { useTransition } from '@/transition'

import NavBarVue from '@/components/NavBar.vue'

const projectsTransition = useTransition('Projects', 0.5)

const canvas: Ref<null | HTMLCanvasElement> = ref(null)

const shapeSize = 200
const shapeSpeed = 5
const transitioningShapeSpeed = 300
const fadeIn = 1
const rotationSpeed = 1
const lineWidth = 10

function randomInteger(min: number, max: number) {
  return Math.floor(Math.random() * (max - min) + min)
}

function randomShape() {
  if (!canvas.value)
    return {
      points: randomInteger(3, 7),
      x: randomInteger(-100, 100),
      y: randomInteger(-100, 100),
      distance: 60,
      opacity: 0,
      rotation: Math.random() * Math.PI * 2,
      rotationSpeed: Math.random() * 2 - 1,
    }

  return {
    points: randomInteger(3, 7),
    x: (randomInteger(-100, 100) * (canvas.value.width | 0)) / 500,
    y: (randomInteger(-100, 100) * (canvas.value.height | 0)) / 500,
    distance: 60,
    opacity: 0,
    rotation: Math.random() * Math.PI * 2,
    rotationSpeed: Math.random() * 2 - 1,
  }
}

onMounted(() => {
  const context = canvas.value?.getContext('2d')

  if (!canvas.value) return

  canvas.value.width = canvas.value?.offsetWidth
  canvas.value.height = canvas.value?.offsetHeight

  addEventListener('resize', () => {
    if (!canvas.value) return

    canvas.value.width = canvas.value?.offsetWidth
    canvas.value.height = canvas.value?.offsetHeight
  })

  let shapes: {
    points: number
    x: number
    y: number
    distance: number
    opacity: number
    rotation: number
    rotationSpeed: number
  }[] = []

  for (let i = 0; i < 100; i++) {
    shapes.push(randomShape())

    shapes[i].distance = randomInteger(0, 60)
    shapes[i].opacity = 1
  }

  let lastTime = -1

  function drawFrame(timestamp: DOMHighResTimeStamp) {
    if (!canvas.value) return
    if (!context) return

    const deltaTime = (timestamp - lastTime) / 1000

    if (lastTime == -1) {
      lastTime = timestamp

      window.requestAnimationFrame(drawFrame)

      return
    }

    lastTime = timestamp

    if (deltaTime > 0.1) {
      window.requestAnimationFrame(drawFrame)

      return
    }

    const centerX = canvas.value.width / 2
    const centerY = canvas.value.height / 2

    context.clearRect(0, 0, canvas.value.width, canvas.value.height)

    for (let shapeIndex = 0; shapeIndex < shapes.length; shapeIndex++) {
      let shape = shapes[shapeIndex]

      shape.distance -=
        (projectsTransition.transitioning.value
          ? shapeSpeed +
            (transitioningShapeSpeed - shapeSpeed) *
              projectsTransition.progress.value
          : shapeSpeed) * deltaTime
      shape.opacity += fadeIn * deltaTime
      shape.rotation += shape.rotationSpeed * rotationSpeed * deltaTime

      if (projectsTransition.transitioning.value && shape.distance < 0) continue

      if (shape.distance < 0) {
        shapes[shapeIndex] = randomShape()
        shape = shapes[shapeIndex]
      }

      const renderSize = shapeSize / shape.distance

      const renderX = centerX + (shape.x / shape.distance) * 90
      const renderY = centerY + (shape.y / shape.distance) * 90

      const sideAngle = (Math.PI * 2) / shape.points

      context.strokeStyle = `rgba(255, 255, 255, ${shape.opacity})`
      context.lineWidth = lineWidth * (renderSize / shapeSize)
      context.lineCap = 'round'

      context.beginPath()

      context.moveTo(
        Math.cos(shape.rotation) * renderSize + renderX,
        Math.sin(shape.rotation) * renderSize + renderY
      )

      for (let side = 1; side <= shape.points; side++) {
        context.lineTo(
          Math.cos(side * sideAngle + shape.rotation) * renderSize + renderX,
          Math.sin(side * sideAngle + shape.rotation) * renderSize + renderY
        )
      }

      context.stroke()
    }

    window.requestAnimationFrame(drawFrame)
  }

  window.requestAnimationFrame(drawFrame)
})
</script>

<style scoped>
#page {
  background-color: var(--main);
}

#wrapper {
  width: 100vw;
  height: 100vh;

  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;

  position: relative;

  overflow: hidden;
}

canvas {
  position: absolute;

  width: 100vw;
  height: 100vh;
}

h1 {
  display: inline-block;

  font-weight: 700;
  font-size: xxx-large;

  margin-bottom: 5rem;

  transition: color 200ms ease-in-out, text-shadow 200ms ease-in-out,
    opacity 200ms ease-in, scale 200ms ease-in;
}

h1:hover {
  color: var(--main);
  text-shadow: -1px 1px 0 var(--text), 1px 1px 0 var(--text),
    1px -1px 0 var(--text), -1px -1px 0 var(--text);
}

h1.transitioning {
  opacity: 0;
  scale: 2;
}

button {
  background-color: var(--grab);
  border-radius: 5px;
  box-shadow: 0 0 1.5rem 0.1rem var(--grab);

  padding: 0.25rem 0.5rem;

  transition: transform 200ms ease-in-out, opacity 200ms ease-in;
}

button:hover {
  transform: scale(1.05);
}

button.transitioning {
  opacity: 0;
}
</style>

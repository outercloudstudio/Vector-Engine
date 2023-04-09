<template>
  <NavBar leftIcon="dataset" leftLink="Projects" />

  <div id="page">
    <div id="main-split">
      <div id="half-vertical-split">
        <div id="half-horizontal-split">
          <Preview />

          <Sidemenu ref="sideMenu" />
        </div>

        <Timeline />
      </div>

      <div id="side-bar">
        <span
          class="material-symbols-outlined icon-button"
          @click="() => sideMenu?.open('export')"
        >
          save
        </span>

        <span
          class="material-symbols-outlined icon-button"
          @click="() => sideMenu?.open('settings')"
        >
          settings
        </span>

        <span
          class="material-symbols-outlined icon-button"
          @click="() => sideMenu?.open('inspector')"
        >
          colorize
        </span>

        <span
          class="material-symbols-outlined icon-button"
          @click="() => sideMenu?.open('console')"
        >
          terminal
        </span>
      </div>
    </div>
  </div>

  <AlertPopup
    :text="EngineStore.blockingErrors[0]"
    :display="EngineStore.blockingErrors.length > 0"
  />
</template>

<script setup lang="ts">
import NavBar from './components/NavBar.vue'
import Timeline from './components/workspace/Timeline.vue'
import Preview from './components/workspace/Preview.vue'
import Sidemenu from './components/workspace/Sidemenu.vue'
import AlertPopup from './components/popups/AlertPopup.vue'

import { ref, onMounted, Ref } from 'vue'
import { useEngineStore } from './stores/EngineStore'

const props = defineProps(['project', 'data'])

const EngineStore = useEngineStore()

const sideMenu: Ref<null | typeof Sidemenu> = ref(null)

onMounted(async () => {
  if (import.meta.hot) {
    import.meta.hot.send('vector-engine:load')

    import.meta.hot.on('vector-engine:load', async (data: any) => {
      await EngineStore.makeEngine(props.project, data)
    })

    import.meta.hot.on('vector-engine:update', async (project: any) => {
      await EngineStore.remakeEngine(props.project)
    })
  }

  // window.addEventListener('project', async project => {
  //   await EngineStore.makeEngine(
  //     (<CustomEvent>project).detail.project,
  //     (<CustomEvent>project).detail.data
  //   )
  // })

  // if (import.meta.env.DEV) {
  //   ;(await import('./dev/dev')).default()
  // }

  // window.addEventListener('project-update', async project => {
  //   await EngineStore.remakeEngine((<CustomEvent>project).detail)
  // })
})
</script>

<style scoped>
#page {
  width: 100%;
  height: 100vh;

  background-color: var(--main);

  box-sizing: border-box;
  padding-top: 2rem;
}

#main-split {
  display: flex;

  flex-direction: row;

  height: 100%;
}

#side-bar {
  border-left: solid 1px var(--secondary);

  min-width: 2rem;

  display: flex;
  flex-direction: column;

  justify-content: center;
  align-items: center;

  gap: 0.5rem;
}

#half-vertical-split {
  display: flex;
  flex-direction: column;

  flex-grow: 1;

  height: 100%;
}

#half-horizontal-split {
  flex-grow: 1;

  display: flex;
}
</style>

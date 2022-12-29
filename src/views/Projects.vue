<template>
    <h1>Projects</h1>

    <button @click="chooseEngineFolder">Choose Folder</button>

    <input v-model="newProjectName"/>

    <button @click="createProject">Create Project</button>

    <button v-for="project in projects" @click="() => loadProject(project)">{{ project.name }}</button>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import router from '@/router'
import { get, set, keys, clear } from 'idb-keyval'
import { getPermissions } from '@/FSUtils'
import { useProjectStore, isDirAProject, doesFolderExistOnFolderHandle } from '@/stores/ProjectStore'
import { useSettingsStore } from '@/stores/SettingsStore'

const ProjectStore = useProjectStore()
const SettingsStore = useSettingsStore()

let projects = ref([])

let newProjectName = ref('New Project')

async function chooseEngineFolder(){
    const dirHandle = await window.showDirectoryPicker()

    if(!(await getPermissions(dirHandle))) return

    await set('engine-folder', dirHandle)

    await loadProjects()
}

async function loadProject(dir: any){
    await ProjectStore.setupProject(dir)
    
    router.push({ name: 'Editor' })
}

async function loadProjects(){
    const dbKeys = await keys()
    
    if(!dbKeys.includes('engine-folder')) return

    projects.value = []
    
    const dir = await get('engine-folder')

    if(!(await getPermissions(dir))) return

    await SettingsStore.load(dir)

    for await (const entry of dir.values()) {    
        if(await isDirAProject(entry)) projects.value.push(entry)
    }
}

async function createProject(){
    const dbKeys = await keys()
    
    if(!dbKeys.includes('engine-folder')) return

    const dir = await get('engine-folder')
    
    if(!(await getPermissions(dir))) return

    if(await doesFolderExistOnFolderHandle(newProjectName.value, dir)) return

    const projectDir = await dir.getDirectoryHandle(newProjectName.value, { create: true })

    await ProjectStore.createProject(projectDir)
    
    router.push({ name: 'Editor' }) 
}

onMounted(async () => {
    await loadProjects()
})
</script>

<style scoped>
    
</style>
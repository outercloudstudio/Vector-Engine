import { defineStore } from 'pinia'
import { ref, computed, watch, toRaw } from 'vue'
import { get, set, keys, clear } from 'idb-keyval'
import { doesFileExistOnFolderHandle, writeFile, getFileText, isJSON, getPermissions } from '@/FSUtils'

const defaultSettings = {
    volume: 100
}

export const useSettingsStore = defineStore('SettingsStore', () => {
    let settings = ref(JSON.parse(JSON.stringify(defaultSettings)))

    let handle = ref(null)

    let shouldBeRecovered = ref(true)

    let settingsVersion = ref(0)

    let isDirty = ref(false)
    let isSaving = ref(false)

    async function load(dir: any){
        console.log('Loading Settings!')
        
        handle.value = dir

        const file = await dir.getFileHandle('editor.json', { create: true })
        
        settings.value = JSON.parse(JSON.stringify(defaultSettings))
        
        const settingsString = await getFileText(file)
    
        if(isJSON(settingsString)) {
            const settingsJSON = JSON.parse(settingsString)

            if(settingsJSON.volume != undefined && typeof settingsJSON.volume == 'number' && settingsJSON.volume >= 0 && settingsJSON.volume <= 100){
                settings.value.volume = settingsJSON.volume
            }
        }

        settingsVersion.value++

        await save()

        await cacheState()

        console.log('Done loading!')
    }

    async function save(){
        console.log('Saving Settings!')

        isSaving.value = true

        const file = await handle.value.getFileHandle('editor.json', { create: true })
        
        await writeFile(file, JSON.stringify(settings.value, null, 4))

        isSaving.value = false

        if(isDirty.value){
            isDirty.value = false

            await save()
        }
    }

    async function safeSave(){
        isDirty.value = true
        
        if(isSaving.value) return

        await save()
    }

    async function cacheState() {
        console.log('Caching Settings!')

        shouldBeRecovered.value = false

        await set('settings-cache', toRaw(settings.value))
        await set('settings-handle-cache', toRaw(handle.value))
    }

    async function reload() {
        console.log('Reloading Settings!')

        await load(handle.value)
    }

    async function recover() {
        if (!shouldBeRecovered.value) return

        console.log('Recovering Settings!')

        // @ts-ignore
        settings.value = await get('settings-cache')
        // @ts-ignore
        handle.value = await get('settings-handle-cache')

        await getPermissions(handle.value)

        await load(handle.value)
    }

    async function updateVolume(volume: number){
        settings.value.volume = volume
        
        await safeSave()
    }
    
    return { settingsVersion, updateVolume, settings, load, reload, recover }
})
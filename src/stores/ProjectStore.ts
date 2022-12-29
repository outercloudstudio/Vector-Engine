import { defineStore } from 'pinia'
import { ref, computed, toRaw, watch } from 'vue'
import { get, set, keys, clear } from 'idb-keyval'
import { doesFileExistOnFolderHandle, writeFile, getFileText, isJSON, getPermissions, doesExist } from '@/FSUtils'
import { useSettingsStore } from '@/stores/SettingsStore'
import cacheProjectWorker from '@/workers/cacheProject?worker'

const defaultScene = `const element = new Element(Builder.Rect)

addElement(element)

yield *
animate(1, Mode.Ease, time => {
element.position.x = time * 1920
})
`

const defaultConfigJSON = {
    initialScene: 'Scene.js',
    contentInference: true,
    frameRate: 60,
    animationLength: 60,
    markers: []
}

const defaultConfig = JSON.stringify(defaultConfigJSON, null, 2)

const audioFormats = {
    mp3: 'mpeg',
    mp4: 'mp4',
    wav: 'wav',
    ogg: 'ogg',
    webm: 'webm'
}

export async function doesFolderExistOnFolderHandle(name: string, folder: any) {
    try {
        await folder.getDirectoryHandle(name)
    } catch (err) {
        return false
    }

    return true
}

export async function isDirAProject(dir: any) {
    try {
        await dir.getFileHandle('.vectorEngine')
    } catch (err) {
        return false
    }

    return true
}

function bufferToString(buffer: ArrayBuffer) {
    return (new TextDecoder('utf-8')).decode(buffer)
}

export const useProjectStore = defineStore('ProjectStore', () => {
    const vfs = ref(null)

    const handle = ref(null)

    const shouldBeRecovered = ref(true)

    const scenesError = ref(null)
    const configError = ref(null)
    const initialSceneError = ref(null)
    const masterAudioTrackError: any = ref(null)

    const vfsVersion = ref(0)

    const masterAudioTrack: any = ref(new Audio())
    const masterAudioTrackBlobURL: any = ref(null)
    const masterAudioTrackVolume: any = ref(null)

    const settingsStore = useSettingsStore()

    async function fileSystemToObject(file: any) {
        let object: any = {
            name: file.name,
            content: null,
            kind: 'file'
        }

        if (file.kind == 'directory') {
            object.content = []
            object.kind = 'folder'

            for await (const entry of file.values()) {
                object.content.push(await fileSystemToObject(entry))
            }
        } else {
            object.content = await (await file.getFile()).arrayBuffer()
        }

        return object
    }

    function getAssets() {
        return getFolder('Assets', vfs.value).content
    }

    function isValidSceneFile(file: any) {
        if (file.kind != 'file') return false

        if (!file.name.endsWith('.js')) return false

        return true
    }

    async function validateProject(dir: any) {
        console.log(`Validating project folder ${dir.name}`)

        const scenesDir = await dir.getDirectoryHandle('Scenes', { create: true })

        let validSceneExists = false
        let validSceneNames = []

        for await (const entry of scenesDir.values()) {
            if (isValidSceneFile(entry)) {
                validSceneExists = true
                validSceneNames.push(entry.name)

                break
            }
        }

        if (!validSceneExists) {
            const sceneFile = await scenesDir.getFileHandle('Scene.js', { create: true })

            const writable = await sceneFile.createWritable()

            await writable.write(defaultScene)

            await writable.close()

            validSceneNames.push(sceneFile.name)
        }

        if (!(await doesFileExistOnFolderHandle('project.json', dir))) {
            const configFile = await dir.getFileHandle('project.json', { create: true })

            await writeFile(configFile, defaultConfig)
        }

        await dir.getDirectoryHandle('Assets', { create: true })
        await dir.getDirectoryHandle('Exports', { create: true })
        await dir.getFileHandle('.vectorEngine', { create: true })
    }

    async function setupProject(dir: any) {
        console.log(`Setting up VFS from ${dir.name}`)

        await validateProject(dir)

        vfs.value = await fileSystemToObject(dir)
        handle.value = dir

        checkForErrors()

        await cacheState()

        vfsVersion.value++
    }

    async function createProject(dir: any) {
        console.log(`Creating VFS from ${dir.name}`)

        await validateProject(dir)

        vfs.value = await fileSystemToObject(dir)
        handle.value = dir

        checkForErrors()

        await cacheState()

        vfsVersion.value++
    }

    function getFolder(name: string, from: any) {
        return from.content.find((item: any) => item.name == name && item.kind == 'folder')
    }

    function getFile(name: string, from: any) {
        return from.content.find((item: any) => item.name == name && item.kind == 'file')
    }

    function checkForScenesError() {
        if (vfs.value == null) {
            // @ts-ignore
            scenesError.value = 'Unexpected error: 1'

            return
        }

        const scenesFolder = getFolder('Scenes', vfs.value)

        if (scenesFolder == undefined) {
            // @ts-ignore
            scenesError.value = 'Missing a scenes folder!'

            return
        }

        scenesError.value = null
    }

    function checkForConfigError() {
        if (vfs.value == null) {
            // @ts-ignore
            configError.value = 'Unexpected error: 2'

            return
        }

        const configFile = getFile('project.json', vfs.value)

        if (configFile == undefined) {
            // @ts-ignore
            configError.value = 'Missing the project.json file!'

            return
        }

        const configText = bufferToString(configFile.content)

        if (!isJSON(configText)) {
            // @ts-ignore
            configError.value = 'The project.json file has invalid JSON!'

            return
        }

        const config = JSON.parse(configText)

        if (config.frameRate == undefined || typeof config.frameRate != 'number') {
            // @ts-ignore
            configError.value = 'The frameRate property of the project.json file must be a number valid number!'

            return
        }

        if (config.frameRate <= 0) {
            // @ts-ignore
            configError.value = 'Frame rate must be a number greater than 0!'

            return
        }

        if (config.animationLength == undefined || typeof config.animationLength != 'number') {
            // @ts-ignore
            configError.value = 'The animationLength property of the project.json file must be a valid number!'

            return
        }

        if (config.animationLength <= 0) {
            // @ts-ignore
            configError.value = 'Animation length must be a number greater than 0!'

            return
        }

        if (config.markers == undefined || !Array.isArray(config.markers)) {
            // @ts-ignore
            configError.value = 'The markers property of the project.json file must be an array!'

            return
        }

        let foundNames: string[] = []

        for (const marker of config.markers) {
            if (marker.name == undefined || typeof marker.name != 'string' || marker.frame == undefined || typeof marker.frame != 'number') {
                // @ts-ignore
                configError.value = 'Markers must contain a valid string name and number frame!'

                return
            }

            if (marker.name == '') {
                // @ts-ignore
                configError.value = 'Markers must not have empty names!'

                return
            }

            if (marker.frame < 0) {
                // @ts-ignore
                configError.value = 'Markers must be of frame 0 or higher!'

                return
            }

            if (foundNames.includes(marker.name)) {
                // @ts-ignore
                configError.value = 'Markers can not share duplicate names!'

                return
            }

            foundNames.push(marker.name)
        }

        if (config.masterAudioTrack != undefined) {
            if (typeof config.masterAudioTrack != 'string') {
                // @ts-ignore
                configError.value = 'Invalid master audio track!'

                return
            }

            if (typeof config.masterAudioTrack != 'string') {
                // @ts-ignore
                configError.value = 'Invalid master audio track!'

                return
            }

            if (getFile(config.masterAudioTrack, getFolder('Assets', vfs.value)) == undefined) {
                // @ts-ignore
                configError.value = 'Specified master audio track file does not exist!'
    
                return
            }
        }

        configError.value = null
    }

    function checkForInitialSceneError() {
        if (config.value.initialScene == undefined) {
            // @ts-ignore
            initialSceneError.value = 'The project.json file is missing the initialScene property!'

            return
        }

        const scene = scenes.value.find((s: any) => s.name == config.value.initialScene)

        if (scene == undefined) {
            // @ts-ignore
            initialSceneError.value = 'Intial scene is an invalid scene!'

            return
        }

        initialSceneError.value = null
    }

    function checkForErrors() {
        checkForScenesError()
        checkForConfigError()
        checkForInitialSceneError()
    }

    const scenes = computed(() => {
        if (vfs.value == null) return []

        let returnScenes = []

        const scenesFolder = getFolder('Scenes', vfs.value)

        if (scenesFolder == undefined) return []

        for (let i = 0; i < scenesFolder.content.length; i++) {
            if (!isValidSceneFile(scenesFolder.content[i])) continue

            returnScenes.push({
                name: scenesFolder.content[i].name,
                content: bufferToString(scenesFolder.content[i].content)
            })
        }

        return returnScenes
    })

    const config = computed(() => {
        if (vfs.value == null) return defaultConfigJSON

        const configFile = getFile('project.json', vfs.value)

        if (configFile == undefined) return defaultConfigJSON

        const configText = bufferToString(configFile.content)

        if (!isJSON(configText)) return defaultConfigJSON

        return JSON.parse(configText)
    })

    const initialScene = computed(() => {
        if (config.value.initialScene == undefined) return ''

        const scene = scenes.value.find((s: any) => s.name == config.value.initialScene)

        if (scene == undefined) return ''

        return scene.content
    })

    const initialSceneName = computed(() => {
        if (config.value.initialScene == undefined) return ''

        const scene = scenes.value.find((s: any) => s.name == config.value.initialScene)

        if (scene == undefined) return ''

        return scene.name
    })

    const error = computed(() => {
        if (scenesError.value != null) return scenesError.value

        if (configError.value != null) return configError.value

        if (initialSceneError.value != null) return initialSceneError.value

        if (masterAudioTrackError.value != null) return masterAudioTrackError.value

        return null
    })

    const frameRate = computed(() => {
        if (config.value.frameRate == undefined) return 60

        if (config.value.frameRate <= 0) return 60

        return config.value.frameRate
    })

    const animationLength = computed(() => {
        if (config.value.animationLength == undefined) return 60

        if (config.value.animationLength <= 0) return 60

        return config.value.animationLength
    })

    const markers = computed(() => {
        if (config.value.markers == undefined) return []

        return config.value.markers
    })

    const masterAudioBuffer = computed(() => {
        if (config.value.masterAudioTrack == undefined) return null

        const file = getFile(config.value.masterAudioTrack, getFolder('Assets', vfs.value))

        if (file == undefined) return null

        return file.content
    })

    watch(masterAudioBuffer, async (val, prevVal) => {
        let audioFormat = 'unkown'

        for (const ending of Object.keys(audioFormats)) {
            if (config.value.masterAudioTrack.endsWith(ending)) {
                audioFormat = audioFormats[ending]

                break
            }
        }

        if (audioFormat == 'unkown') {
            masterAudioTrackError.value = 'Unsupported master audio track format!'

            return
        }

        if (masterAudioTrack.value != null) {
            window.URL.revokeObjectURL(masterAudioTrackBlobURL.value)
        }

        const blob = new Blob([val], { type: `audio/${audioFormat}` })
        const url = window.URL.createObjectURL(blob)

        masterAudioTrack.value.src = url
        masterAudioTrackBlobURL.value = url

        let buffer = undefined

        try {
            const ctx = new AudioContext()

            let dataBuffer = new ArrayBuffer(masterAudioBuffer.value.byteLength)
            new Uint8Array(dataBuffer).set(new Uint8Array(masterAudioBuffer.value))

            buffer = await ctx.decodeAudioData(dataBuffer)
        } catch {
            masterAudioTrackError.value = 'Error readng master audio track!'

            return
        }

        let pcm = []

        let min = 9999
        let max = -9999

        let data = []

        for (let i = 0; i < buffer.numberOfChannels; i++) {
            data.push(buffer.getChannelData(i))
        }

        const samplesPerFrame = buffer.sampleRate / frameRate.value

        for (let i = 0; i < data[0].length && i < samplesPerFrame * animationLength.value; i++) {
            pcm[i] = 0

            for (let j = 0; j < data.length; j++) {
                pcm[i] += data[j][i]
            }

            pcm[i] /= data.length
        }

        let volume: number[] = []

        for (let i = 0; i < animationLength.value; i++) {
            volume[i] = 0

            for (let j = 0; j < samplesPerFrame; j++) {
                if (pcm[i * samplesPerFrame + j] == undefined) {
                    // console.warn(`Pcm ${i * samplesPerFrame} + ${j} (${i * samplesPerFrame + j} is undefined)`)
                    break
                }

                volume[i] = Math.max(pcm[i * samplesPerFrame + j], volume[i])
            }

            if (min > volume[i]) min = volume[i]
            if (max < volume[i]) max = volume[i]
        }

        for (let i = 0; i < animationLength.value; i++) {
            volume[i] = Math.pow(volume[i] / max, 1.5)
        }

        masterAudioTrackError.value = null

        masterAudioTrackVolume.value = volume
    })

    watch(() => settingsStore.settings.volume, (val, preVal) => {
        masterAudioTrack.value.volume = val / 100
    })

    async function cacheState() {
        console.log('Caching Project...')
        
        const cacheWorker = new cacheProjectWorker()
        cacheWorker.postMessage([toRaw(vfs.value), toRaw(handle.value)])

        shouldBeRecovered.value = false
    }

    async function reload() {
        console.log('Reloading Project!')

        if(!await doesExist(handle.value)) {
            return false
        }

        await setupProject(handle.value)

        return true
    }

    async function recover() {
        if (!shouldBeRecovered.value) return true

        console.log('Recovering Project!')

        // @ts-ignore
        vfs.value = await get('project-vfs-cache')
        // @ts-ignore
        handle.value = await get('project-handle-cache')

        await getPermissions(handle.value)

        if(!await doesExist(handle.value)) {
            return false
        }
        
        await setupProject(handle.value)

        return true
    }

    async function newMarker(frame: number) {
        let newMarkerName = 'marker1'
        let markerIndex = 1

        while (markers.value.find((marker: {
            name: string,
            frame: number
        }) => marker.name == newMarkerName)) {
            markerIndex++

            newMarkerName = `marker${markerIndex}`
        }

        const configJSON = config.value

        configJSON.markers.push({
            name: newMarkerName,
            frame
        })

        const content = JSON.stringify(configJSON, null, 2)

        // @ts-ignore
        await writeFile(await handle.value.getFileHandle('project.json'), content)

        // @ts-ignore
        getFile('project.json', vfs.value).content = new TextEncoder().encode(content)

        vfsVersion.value++

        await cacheState()
    }

    async function renameMarker(prevName: string, name: string) {
        const configJSON = config.value

        const index = configJSON.markers.findIndex((marker: any) => marker.name == prevName)

        configJSON.markers[index].name = name

        const content = JSON.stringify(configJSON, null, 2)

        // @ts-ignore
        await writeFile(await handle.value.getFileHandle('project.json'), content)

        // @ts-ignore
        getFile('project.json', vfs.value).content = new TextEncoder().encode(content)

        vfsVersion.value++

        await cacheState()
    }

    async function reposMarker(name: string, frame: number) {
        const configJSON = config.value

        const index = configJSON.markers.findIndex((marker: any) => marker.name == name)

        configJSON.markers[index].frame = frame

        const content = JSON.stringify(configJSON, null, 2)

        // @ts-ignore
        await writeFile(await handle.value.getFileHandle('project.json'), content)

        // @ts-ignore
        getFile('project.json', vfs.value).content = new TextEncoder().encode(content)

        vfsVersion.value++

        await cacheState()
    }

    async function deleteMarker(name: string) {
        const configJSON = config.value

        const index = configJSON.markers.findIndex((marker: any) => marker.name == name)

        configJSON.markers.splice(index, 1)

        const content = JSON.stringify(configJSON, null, 2)

        // @ts-ignore
        await writeFile(await handle.value.getFileHandle('project.json'), content)

        // @ts-ignore
        getFile('project.json', vfs.value).content = new TextEncoder().encode(content)

        vfsVersion.value++

        await cacheState()
    }

    async function setupImageSequenceExport(name: string){
        const exportsDir = await handle.value.getDirectoryHandle('Exports')
        
        if(await doesFolderExistOnFolderHandle(name, exportsDir)){
            await exportsDir.removeEntry(name, { recursive: true })
        }

        return await exportsDir.getDirectoryHandle(name, { create: true })
    }

    return { vfs, getAssets, setupImageSequenceExport, masterAudioTrackVolume, masterAudioTrack, deleteMarker, renameMarker, reposMarker, newMarker, markers, vfsVersion, frameRate, animationLength, error, createProject, setupProject, scenes, config, initialScene, initialSceneName, reload, recover }
})

<template>
    <div id="menu-container">
        <div
            id="menu"
            :class="{ open: menuOpen != 'none' }"
        >
            <div id="export-menu" v-show="menuOpen == 'export'">
                <p class="label">Export Name</p>
                <input 
                    v-model.lazy="exportName"
                    @blur="exportName = $event.target.value"
                />
                
                <!--<p class="label">Export Format</p>
                <Dropdown :options="['Image Sequence']" v-model="format" />

                <p class="label">Export Quality</p>
                <Dropdown :options="['Standard']" v-model="exportQuality" />-->

                <p class="label">In / Out Frames</p>
                <div id="in-out-container">
                    <input 
                        v-model.lazy="inFrameBuffer"
                        @blur="inFrameBuffer = $event.target.value"
                    />

                    <input 
                        v-model.lazy="outFrameBuffer"
                        @blur="outFrameBuffer = $event.target.value"
                    />
                </div>

                <button @click="doExport">
                    Export
                </button>
            </div>

            <div id="marker-menu" v-show="menuOpen == 'marker' && selectedMarker != ''">
                <p class="label">Marker Name</p>
                <input 
                    v-model.lazy="markerNameBuffer"
                    @blur="markerNameBuffer = $event.target.value"
                />

                <p class="label">Marker Frame</p>
                <input 
                    v-model.lazy="markerFrameBuffer"
                    @blur="markerFrameBuffer = $event.target.value"
                />

                <div id="frame-buttons-container">
                    <button @click="deleteMarker">Delete</button>
    
                    <button @click="emit('update:selectedMarker', '')">Unselect</button>
                </div>
            </div>

            <div id="editor-menu" v-show="menuOpen == 'editor'">
                <p class="label">Editor Volume</p>
                <input type="range" min="0" max="100" id="editor-volume" :value="SettingsStore.settings.volume" @input="(volume) => SettingsStore.updateVolume(parseInt(volume.target.value))">
            </div>
        </div>

        <div id="sidebar">
            <span 
                class="material-symbols-rounded"
                id="export-button"
                @click="() => openMenu('export')"
            >download</span>

            <span 
                class="material-symbols-rounded"
                id="export-button"
                @click="() => openMenu('marker')"
            >dataset</span>

            <span 
                class="material-symbols-rounded"
                id="export-button"
                @click="() => openMenu('editor')"
            >settings</span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import Dropdown from '@/components/Dropdown.vue'
import { writeFile } from '@/FSUtils'
import { useProjectStore } from '@/stores/ProjectStore'
import { useSettingsStore } from '@/stores/SettingsStore'

const ProjectStore = useProjectStore()
const SettingsStore = useSettingsStore()

const props = defineProps(['canvas', 'currentFrame', 'selectedMarker'])
const emit = defineEmits(['updateFrame', 'update:selectedMarker'])

let markerName = ref('Strange Marker')
let markerFrame = ref(0)

let inFrame = ref(0)
let outFrame = ref(ProjectStore.animationLength)

let markerNameBuffer = computed({
  get: () => markerName.value,
  set: (value) => {
    console.log(value)
      
    if(value != '' && ProjectStore.markers.find(marker => marker.name == value) == undefined){
        markerName.value = value
    }else{
        const cachedValue = markerName.value

        markerName.value = ''

        markerName.value = cachedValue
    }
  }
})

let markerFrameBuffer = computed({
  get: () => markerFrame.value,
  set: (frame) => {
    if(/^[0-9]+$/.test(frame)){
        markerFrame.value = clamp(parseInt(frame), 0, ProjectStore.animationLength - 1)
    }else{
        const val = markerFrame.value
        
        markerFrame.value = -1
        
        markerFrame.value = val
    }
  }
})

let inFrameBuffer = computed({
  get: () => inFrame.value,
  set: (frame) => {
    if(/^[0-9]+$/.test(frame)){
        inFrame.value = clamp(parseInt(frame), 0, Math.min(ProjectStore.animationLength - 1, outFrame.value - 1))
    }else{
        const val = inFrame.value
        
        inFrame.value = -1
        
        inFrame.value = val
    }
  }
})

let outFrameBuffer = computed({
  get: () => outFrame.value,
  set: (frame) => {
    if(/^[0-9]+$/.test(frame)){
        outFrame.value = clamp(parseInt(frame), Math.max(1, inFrame.value + 1), ProjectStore.animationLength)
    }else{
        const val = outFrame.value
        
        outFrame.value = -1
        
        outFrame.value = val
    }
  }
})

const clamp = (num, min, max) => Math.min(Math.max(num, min), max)

let dontUpdateName = ref(false)
let dontUpdateFrame = ref(false)

watch(() => props.selectedMarker, (value, prevValue) => {
    if(value == '') return
    
    markerName.value = value
    markerFrame.value = ProjectStore.markers.find(marker => marker.name == value).frame

    dontUpdateName.value = true
    dontUpdateFrame.value = true
})

watch(markerName, async (value, prevValue) => {
    if(dontUpdateName.value) {
        dontUpdateName.value = false
        
        return
    }

    await ProjectStore.renameMarker(prevValue, value)

    emit('update:selectedMarker', value)
})

watch(markerFrame, async (value, prevValue) => {
    if(dontUpdateFrame.value) {
        dontUpdateFrame.value = false
        
        return
    }

    await ProjectStore.reposMarker(props.selectedMarker, value)
})

let menuOpen = ref('none')

function openMenu(menu: string){
    if(menuOpen.value != menu){
        menuOpen.value = menu
    }else{
        menuOpen.value = 'none'
    }    
}

function closeMenu(){
    menuOpen.value = 'none'  
}

function forceOpenMenu(menu: string){
    if(menuOpen.value != menu){
        menuOpen.value = menu
    }
}

function deleteMarker(){
    ProjectStore.deleteMarker(props.selectedMarker)

    emit('update:selectedMarker', '')
}

defineExpose({
    forceOpenMenu,
    closeMenu
})
    
let format = ref('Image Sequence')
let exportName = ref('export')
let exportQuality = ref('Standard')

function doExport(){
    exportImageSequence()
    
    //if(format.value == 'Image Sequence')
}

async function exportImageSequence(){
    const dir = await ProjectStore.setupImageSequenceExport(exportName.value)
    
    emit('updateFrame', 0)

    async function newFrame(){
        return new Promise(async resolve => {
            if(props.currentFrame < inFrame.value){
                console.log(`Skipping frame ${props.currentFrame} out of ${ProjectStore.animationLength}`)
                
                emit('updateFrame', props.currentFrame + 1)

                await nextTick()

                await newFrame()
                
                resolve()
            }else{
                setTimeout(async () => {
                    console.log(`Rendering frame ${props.currentFrame} out of ${ProjectStore.animationLength}`)

                    const blob = await new Promise(res => {
                        canvas.toBlob(blob => {
                            res(blob)
                        })
                    })

                    const file = await dir.getFileHandle(`frame_${props.currentFrame.toString().padStart(ProjectStore.animationLength.toString().length, '0')}.png`, { create: true })
                        
                    await writeFile(file, blob)
            
                    emit('updateFrame', props.currentFrame + 1)
        
                    if(props.currentFrame < ProjectStore.animationLength - 1 && props.currentFrame < outFrame.value - 1) await newFrame()
    
                    resolve()
                }, 1)
            }
        })
    }

    await newFrame()
}
    
async function exportWebm(){
    emit('updateFrame', 0)

    const frame = new VideoFrame(canvas, { timestamp: 0 })
    
    alert(frame)
    
    const init = {
      output: (chunk) => {
        try{
            alert(chunk)

          const buffer = new ArrayBuffer(chunk.byteLength)

          chunk.copyTo(buffer)
        }catch(err){
            alert(err)
        }
      },
      error: (e) => {
        alert(e.message)
      },
    }

    const config = {
      codec: 'vp8',
      width: 1920,
      height: 1080,
      bitrate: 2_000_000, // 2 Mbps
      framerate: 60,
    }

    const encoder = new VideoEncoder(init)
    encoder.configure(config)
    encoder.encode(frame)
    frame.close()

    await encoder.flush()

    alert('Finished!')
    
    // const videoWriter = new window.WebMWriter({
    //     quality: 0.95,
    
    //     frameRate: ProjectStore.frameRate,
    
    //     transparent: false,
    // })

    // emit('updateFrame', 0)

    // async function newFrame(){
    //     return new Promise(async resolve => {
    //         if(props.currentFrame < inFrame.value){
    //             console.log(`Skipping frame ${props.currentFrame} out of ${ProjectStore.animationLength}`)
                
    //             emit('updateFrame', props.currentFrame + 1)

    //             await nextTick()

    //             await newFrame()
                
    //             resolve()
    //         }else{
    //             setTimeout(async () => {
    //                 console.log(`Rendering frame ${props.currentFrame} out of ${ProjectStore.animationLength}`)
                    
    //                 videoWriter.addFrame(canvas)
            
    //                 emit('updateFrame', props.currentFrame + 1)
        
    //                 if(props.currentFrame < ProjectStore.animationLength && props.currentFrame < outFrame.value) await newFrame()
    
    //                 resolve()
    //             }, 1)
    //         }
    //     })
    // }

    // await newFrame()

    // const blob = await videoWriter.complete()

    // const link = document.createElement('a')
    // const url =  window.URL.createObjectURL(blob)
    // link.href = url
    // link.download = `${exportName.value}.webm`
    // link.click()
    // window.URL.revokeObjectURL(url)
}
</script>

<style scoped>
    .label {
        margin-top: 1rem;
        margin-bottom: 0.2rem;
    }

    #menu-container {
        display: flex;
    }

    #menu {
        background: var(--dark);
        
        width: 0rem;
        
        transition: width 0.2s ease-in-out;

        overflow-x: hidden;
    }

    #menu.open{
        width: 16rem;
    }
    
    #menu > * > * {
        margin-left: 1rem;
    }

    #menu > * > button {
        margin-top: 1rem;

        transition: transform 0.2s ease-in-out;

        cursor: pointer
    }

    #menu > * > button:hover {
        transform: scale(1.2);
    }

    #menu > * > input {
        background: var(--medium);

        padding: 0.4rem;

        display: table;
    }

    #menu > * > input[type="range"] {
        background: none;
        margin-left: 0.6rem;
    }

    #menu > * > * > input {
        background: var(--medium);

        padding: 0.4rem;

        display: table;
    }

    #export-name {
        width: 10rem;
    }

    #in-out-container {
        margin-right: 1rem;
        
        display: flex;
        justify-content: space-between;
    }

    #frame-buttons-container {
        margin-right: 1rem;
        margin-top: 1rem;
        
        display: flex;
        justify-content: space-between;
    }

    #sidebar {
        background: var(--medium);
        
        width: 2.5rem;

        display: flex;
        flex-direction: column;
        justify-content: center;
    }

    #sidebar > * {
        margin-bottom: 0.5rem;
    }
</style>
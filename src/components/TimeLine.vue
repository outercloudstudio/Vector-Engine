<template>
    <canvas
        id="timeline"
        ref="timeline"
        @mousedown.prevent="mouseDown"
        @mousemove="interact"
        @mouseup.prevent="mouseUp"
        @mouseleave="mouseLeave"
        @wheel="scroll"
    ></canvas>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue'
import { useProjectStore } from '@/stores/ProjectStore'

const ProjectStore = useProjectStore()

const props = defineProps(['currentFrame', 'selectedMarker', 'events'])
const emit = defineEmits(['updateFrame', 'update:selectedMarker', 'context'])
    
watch(() => props.currentFrame, (val, prevVal) => {
    if(val < beginX.value){
        const offset = beginX.value - val
        
        beginX.value = clamp(beginX.value - offset, 0, endX.value - 1)
        endX.value = clamp(endX.value - offset, beginX.value + 1, ProjectStore.animationLength)
    }

    if(val > endX.value){
        const offset = val - endX.value
        
        beginX.value = clamp(beginX.value + offset, 0, endX.value - 1)
        endX.value = clamp(endX.value + offset, beginX.value + 1, ProjectStore.animationLength)
    }
    
    render()
})

watch(() => props.events, (val, prevVal) => {
    render()
})

watch(() => ProjectStore.animationLength, (val, prevVal) => {
    render()
})

watch(() => ProjectStore.markers, (val, prevVal) => {
    render()
})

watch(() => props.selectedMarker, (val, prevVal) => {
    isHoldingMarker.value = false
    
    render()
})

watch(() => ProjectStore.masterAudioTrackVolume, (val, prevVal) => {
    render()
})
    
const clamp = (num, min, max) => Math.min(Math.max(num, min), max)

let timeline = ref(null)

let timelineActive = false

let isHoldingMarker = ref(false)
let isHoldingFrame = ref(0)

watch(isHoldingFrame, (val, prevVal) => {
    render()
})

const markerSize = 10
const selectDistance = 8

async function mouseDown(event){
    if(event.which == 1){
        const relX = event.clientX - timeline.value.getBoundingClientRect().left
        const relY = event.clientY - timeline.value.getBoundingClientRect().top

        let picked = false

        let closestMarker = undefined
        let closestMarkerDistance = -1

        let markersAtFrame = []

        for(const marker of ProjectStore.markers){
            if(markersAtFrame[marker.frame] == undefined){
                markersAtFrame[marker.frame] = 0
            }else{
                markersAtFrame[marker.frame]++
            }
            
            const markerX = frameToRelX(marker.frame) - markerSize / 2
            const markerY = 70 + markerSize / 2 + 15 * markersAtFrame[marker.frame]

            const dist = Math.sqrt(Math.pow(relX - markerX, 2) + Math.pow(relY - markerY, 2))
            
            if((closestMarker == undefined || dist < closestMarkerDistance) && dist <= selectDistance){
                closestMarkerDistance = dist
                closestMarker = marker.name
            }
        }

        if(closestMarker != undefined){
            emit('update:selectedMarker', closestMarker)

            await nextTick()

            isHoldingMarker.value = true

            return
        }
        
        timelineActive = true
    
        interact(event)
    }else{
        emit('context', event.clientX, event.clientY)
    }
}

function mouseLeave(){
    isHoldingMarker.value = false

    timelineActive = false
}

async function mouseUp(event){
    if(event.which != 1) return 

    if(isHoldingMarker.value){
        const frame = screenXToFrame(event.clientX)

        isHoldingMarker.value = false

        await ProjectStore.reposMarker(props.selectedMarker, frame)
        
        const val = props.selectedMarker
        
        emit('update:selectedMarker', '')

        await nextTick()

        emit('update:selectedMarker', val)
    }

    timelineActive = false
}
    
function interact(event){ 
    if(timelineActive) emit('updateFrame', screenXToFrame(event.clientX))

    if(isHoldingMarker.value) isHoldingFrame.value = screenXToFrame(event.clientX)
}

function render() {
    const ctx = timeline.value.getContext('2d')

    ctx.fillStyle = '#27333E'
    ctx.fillRect(0, 0, timeline.value.width, timeline.value.height)

    ctx.fillStyle = '#31404E'
    ctx.fillRect(0, 0, timeline.value.width, 30)

    const viewRange = endX.value - beginX.value
    const labelInterval = Math.max(Math.floor(viewRange / 60) * 5, 1)
    const lineInterval = Math.max(Math.floor(viewRange / 60), 1)
    
    for(let i = beginX.value; i < endX.value; i++){
        if(i % labelInterval == 0){
            const x = frameToRelX(i)
            
            if(i != 0){
                ctx.fillStyle = '#FFFFFF'
                ctx.fillRect(x, 10, 2, 20)
            }

            ctx.fillStyle = '#FFFFFF'
            ctx.font = '10px Inter';
            ctx.fillText(i, x + 5, 18)
        }else if(i % lineInterval == 0){
            const x = frameToRelX(i)
            
            ctx.fillStyle = '#FFFFFF'
            ctx.fillRect(x, 25, 2, 5)
        }
    }

    for(event of props.events){
        ctx.fillStyle = '#4B5E70'

        const start = frameToRelX(event.from)
        const end = frameToRelX(event.to)
        
        if(event.kind == 'transition') {
            ctx.fillStyle = '#27333E'
            ctx.fillRect(start, 35, end - start, 30)
            
            const gradient = ctx.createLinearGradient(start, 35, end, 65)
            
            for(let i = 0; i < 100; i++){
                const eased = Math.pow(i / 100, 3)
                
                gradient.addColorStop(eased, `rgba(75, 94, 112, ${eased})`)
            }
            
            ctx.fillStyle = gradient
        }

        ctx.fillRect(start, 35, end - start, 30)

        if(event.name){
            ctx.fillStyle = '#FFFFFF'
            ctx.font = '12px Inter';
            ctx.fillText(event.name, start + 10, 53)
        }
    }

    for(let i = beginX.value; i < endX.value; i++){
        if(ProjectStore.masterAudioTrackVolume == null) break
        
        const x = frameToRelX(i)
        const val = clamp(ProjectStore.masterAudioTrackVolume[i], 0, 1)

        ctx.fillStyle = `rgba(255, 255, 255, 0.3)`
        ctx.fillRect(x, 65, timeline.value.width / (endX.value - beginX.value), 30 * val)
    }

    let markersAtFrame = []

    for(const marker of ProjectStore.markers){
        if(markersAtFrame[marker.frame] == undefined){
            markersAtFrame[marker.frame] = 0
        }else{
            markersAtFrame[marker.frame]++
        }
        
        const markerX = frameToRelX(marker.frame) - markerSize / 2
        const markerY = 70 + markerSize / 2 + 15 * markersAtFrame[marker.frame]
        
        ctx.translate(markerX + markerSize / 2, markerY)
        
        ctx.rotate(45 * Math.PI / 180)

        ctx.translate(-markerX - markerSize / 2, -markerY)

        ctx.fillStyle = '#4B5E70'
        ctx.strokeStyle = '#000000'
        ctx.lineWidth = 1

        ctx.fillRect(markerX, markerY, markerSize, markerSize)

        if(marker.name == props.selectedMarker) {
            ctx.strokeStyle = '#FFFFFF'
            ctx.lineWidth = 2
        }
        
        ctx.strokeRect(markerX, markerY, markerSize, markerSize)

        ctx.resetTransform()
    }

    if(isHoldingMarker.value){
        const markerX = frameToRelX(isHoldingFrame.value) - markerSize / 2
        const markerY = 70 + markerSize / 2
        
        ctx.translate(markerX + markerSize / 2, markerY)
        
        ctx.rotate(45 * Math.PI / 180)

        ctx.translate(-markerX - markerSize / 2, -markerY)

        ctx.fillStyle = 'rgba(255, 255, 255, 0.4)'

        ctx.fillRect(markerX, markerY, markerSize, markerSize)

        ctx.resetTransform()
    }
    
    ctx.fillStyle = '#61afef'
    ctx.fillRect(frameToRelX(props.currentFrame), 0, 2, timeline.value.height)
}

function frameToRelX(frame: number){
    return (frame - beginX.value) / (endX.value - beginX.value) * timeline.value.width
}

function relXToFrame(x: number){
    const width = timeline.value.getBoundingClientRect().width

    return clamp(Math.round(x / width * (endX.value - beginX.value)), 0, (endX.value - beginX.value) - 1) + beginX.value
}

function screenXToFrame(x: number){
    const relX = x - timeline.value.getBoundingClientRect().left

    return relXToFrame(relX)
}

let beginX = ref(0)
let endX = ref(ProjectStore.animationLength)

function scroll(event){
    const scrollX = Math.ceil(event.deltaX / 100)
    const scrollY = Math.ceil(event.deltaY / 100)

    let viewRange = endX.value - beginX.value
    let zoomAmount = Math.ceil(Math.abs(Math.pow(Math.log(viewRange / 60), Math.abs(scrollY)))) * scrollY / Math.abs(scrollY)

    if(zoomAmount == 0) zoomAmount = 1
    
    beginX.value = beginX.value - zoomAmount
    endX.value = endX.value + zoomAmount

    if(scrollX != 0){
        viewRange = endX.value - beginX.value
        
        beginX.value += Math.ceil(scrollX * viewRange / 10)
        endX.value += Math.ceil(scrollX * viewRange / 10)
    }

    beginX.value = clamp(beginX.value, 0, endX.value - 1)
    endX.value = clamp(endX.value, beginX.value + 1, ProjectStore.animationLength)

    render()
}

defineExpose({
    screenXToFrame
})

onMounted(async () => {
    let computedStyle = getComputedStyle(timeline.value)

    let elementHeight = timeline.value.clientHeight
    let elementWidth = timeline.value.clientWidth
    
    elementHeight -= parseFloat(computedStyle.paddingTop) + parseFloat(computedStyle.paddingBottom)
    elementWidth -= parseFloat(computedStyle.paddingLeft) + parseFloat(computedStyle.paddingRight)

    timeline.value.width = elementWidth
    timeline.value.height = elementHeight

    addEventListener('resize', (event) => {
        let computedStyle = getComputedStyle(timeline.value)

        let elementHeight = timeline.value.clientHeight
        let elementWidth = timeline.value.clientWidth
        
        elementHeight -= parseFloat(computedStyle.paddingTop) + parseFloat(computedStyle.paddingBottom)
        elementWidth -= parseFloat(computedStyle.paddingLeft) + parseFloat(computedStyle.paddingRight)
    
        timeline.value.width = elementWidth
        timeline.value.height = elementHeight

        render()
    })

    render()
})
</script>

<style scoped>
    #timeline{
        width: 100%;
        
        height: 8rem;

        background: #ffffff;
    }
</style>
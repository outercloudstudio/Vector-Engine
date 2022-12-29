<template>
    <div id="action-bar">
            <div id="base-controls">
                <input
                    id="frame-number"
                    v-model.lazy="currentFrameBuffer"
                    @blur="currentFrameBuffer = $event.target.value"
                />
                
                <input
                    id="time-number"
                    v-model.lazy="currentTimeBuffer"
                    @blur="currentTimeBuffer = $event.target.value"
                />
            </div>
            
            <div id="base-controls">
                <span 
                    class="material-symbols-rounded"
                    id="reset-button"
                    @click="$emit('reset')"
                >fast_rewind</span>
                
                <span 
                    class="material-symbols-rounded"
                    id="next-button"
                    @click="$emit('prev')"
                >skip_previous</span>
                
                <span 
                    class="material-symbols-rounded"
                    id="play-button"
                    @click="emit('pause')"
                >{{ isPlaying ? 'pause': 'play_arrow' }}</span>

                <span 
                    class="material-symbols-rounded"
                    id="next-button"
                    @click="$emit('next')"
                >skip_next</span>

                <input
                    id="speed-number"
                    v-model.lazy="playbackSpeedBuffer"
                    @blur="playbackSpeedBuffer = $event.target.value"
                />

                <p id="fr-number">{{ ProjectStore.frameRate }}</p>
            </div>

            <p id="anim-length">{{ ProjectStore.animationLength }}</p>
        </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useProjectStore } from '@/stores/ProjectStore'

const ProjectStore = useProjectStore()

const props = defineProps(['currentFrame', 'playbackSpeed', 'isPlaying'])
const emit = defineEmits(['updateFrame', 'update:playbackSpeed', 'pause', 'next', 'prev', 'reset'])

const clamp = (num, min, max) => Math.min(Math.max(num, min), max)
    
let currentFrameBuffer = computed({
  get: () => props.currentFrame,
  set: (frame) => {
    if(/^[0-9]+$/.test(frame)){
        emit('updateFrame', clamp(parseInt(frame), 0, ProjectStore.animationLength - 1))
    }else{
        const val = props.currentFrame
        
        emit('updateFrame', -1)
        
        emit('updateFrame', val)
    }
  }
})

let currentTimeBuffer = computed({
    get: () => {
    const seconds = props.currentFrame / ProjectStore.frameRate
    
    const minutes = Math.floor(seconds / ProjectStore.frameRate)
    
    const leftSeconds = Math.floor((seconds - minutes * ProjectStore.frameRate) * 1000) / 1000

    return `${minutes}:${leftSeconds}`
    },
    set: (frame) => {
        if(/^(([0-9]+:[0-9]+\.[0-9]{1,3})|([0-9]+:[0-9]+))$/.test(frame)){
            const minutes = parseInt(frame.split(':')[0])
            const seconds = parseFloat(frame.split(':')[1]) + minutes * ProjectStore.frameRate
            const frames = Math.floor(seconds * ProjectStore.frameRate)
            
            emit('updateFrame', clamp(frames, 0, ProjectStore.animationLength - 1))
        }else{
            const val = props.currentFrame
            
            emit('updateFrame', -1)
            
            emit('updateFrame', val)
        }
    }
})

let playbackSpeedBuffer = computed({
    get: () => `${props.playbackSpeed}x`,
    set: (speed) => {
        if(/^(([0-9]+x?)|([0-9]+\.[0-9]+x?))$/.test(speed)){
            if(speed.endsWith('x')){
                emit('update:playbackSpeed', Math.max(parseFloat(speed.substring(0, speed.length - 1)), 0.001))
            }else{
                emit('update:playbackSpeed', Math.max(parseFloat(speed), 0.001))
            }
        }else{
            const val = props.playbackSpeed
            
            emit('update:playbackSpeed', -1)
            
            emit('update:playbackSpeed', val)
        }
    }
})
</script>

<style scoped>
    #action-bar{
        width: 100%;
        
        height: 2rem;
        
        margin-top: 1rem;

        background: var(--dark);

        display:flex;

        justify-content: space-between;
        align-items: center;
    }

    #frame-number {
        margin-right: 0.5rem;
    }

    #time-number {
        color: var(--light);
    }

    #speed-number {
        margin-left: 1rem;
    }

    #anim-length {
        width: 8.5rem;
        
        margin-right: 0.5rem;

        text-align: right;
    }

    #base-controls {
        display: flex;
        align-items: center;

        margin-left: 0.5rem;
    }

    #play-button, #next-button, #prev-button, #reset-button {
        transition: transform 0.2s ease-in-out;

        cursor: pointer
    }
    
    #play-button:hover, #next-button:hover, #prev-button:hover, #reset-button:hover {
        transform: scale(1.2);
    }

    #reset-button {
        margin-right: 1rem;
    }
</style>
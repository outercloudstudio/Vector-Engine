<template>
    <div class="window" oncontextmenu="return false" v-show="isOpen" @click="close">
       <div class="popup" ref="popup" :style="{ left: left + 'px', top: top + 'px' }">
            <button @click="newMarker">New Marker</button>
       </div>
    </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'

const emit = defineEmits(['newMarker'])
    
let isOpen = ref(false)
let top = ref(0)
let left = ref(0)
let popup = ref(null)

let clickedX = 0

async function open(x, y) {
    console.log(`Opening context ${x} ${y}`)

    isOpen.value = true

    await nextTick()
    
    left.value = x - 10
    top.value = y - popup.value.offsetHeight + 10

    clickedX = x
}

function close() {
    isOpen.value = false
}

function newMarker(){
    emit('newMarker', clickedX)
}

defineExpose({
  open
})
</script>

<style scoped>
    .window {
        position: absolute;

        left: 0;
        right: 0;
        top: 0;
        bottom: 0;

        z-index: 1;

        overflow: hidden;
    }

    .popup {
        padding: 0.5rem;
        
        display: flex;
        flex-direction: column;

        background: var(--dark);

        position: relative;

        width: 12rem;
    }

    .label {
        margin: 0;
    }

    .message {
        margin: 0;
        margin-top: 1rem;
    }
</style>
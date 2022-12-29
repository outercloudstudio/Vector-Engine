<template>
    <div class="dropdown">
        <div class="choice">
            <p>{{options[selected]}}</p>

            <span class="material-symbols-rounded drop-button" @click="toggleOpen">
                arrow_drop_down
            </span>
        </div>
        
        <div class="options" v-if="open">
            <div v-for="(option, i) in options">
                <p v-if="selected != i" @click="() => choose(i)">{{ option }}</p>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

let selected = ref(0)
let open = ref(false)

const props = defineProps(['options', 'modelValue'])
const emit = defineEmits(['update:modelValue'])

watch(() => props.modelValue, (val, prevVal) => {
    if(props.options.includes(val)){
        selected.value = props.options.indexOf(val)
    }
})

function toggleOpen() {
    open.value = !open.value
}

function choose(index: number){
    selected.value = index

    emit('update:modelValue', props.options[index])
}
</script>

<style scoped>
    .dropdown {
        background: var(--medium);

        width: 10rem;

        padding: 0.4rem;

        display: flex;
        flex-direction: column;
        justify-content: flex-start;
    }

    .choice {
        height: 1rem;

        display: flex;

        align-items: center;
    }

    .choice > p {
        margin: 0;

        flex-grow: 1;

        font-size: 0.75rem;
    }

    .options {
        margin-top: 0.2rem;

        border-top: 2px solid var(--dark);
    }

    .options > * > p {
        margin: 0;
        
        margin-top: 0.5rem;

        font-size: 0.75rem;
    }

    .drop-button {
        transition: transform 0.2s ease-in-out;

        cursor: pointer
    }
    
    .drop-button:hover {
        transform: scale(1.2);
    }
</style>
<template>
    <div class="window">
       <div class="popup">
            <h3 class="label">There was an error!</h3>
            <p class="message">{{ message }}</p>
       </div>
    </div>
</template>

<script setup lang="ts">
const props = defineProps(['message'])
</script>

<style scoped>
    .window {
        position: absolute;

        left: 0;
        right: 0;
        top: 0;
        bottom: 0;

        background: var(--overlay);

        display: flex;
        justify-content: center;
        align-items: center;

        backdrop-filter: blur(8px);

        z-index: 2
    }

    .popup {
        padding: 1rem;

        border-radius: 0.5rem;
        
        display: flex;
        flex-direction: column;

        background: var(--dark);
    }

    .label {
        margin: 0;
    }

    .message {
        margin: 0;
        margin-top: 1rem;
    }
</style>
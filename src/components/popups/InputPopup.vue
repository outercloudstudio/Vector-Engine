<template>
  <Transition name="fade">
    <div v-if="display" id="component">
      <div id="popup">
        <p>{{ text }}</p>

        <input v-model="inputValue" :placeholder="placeholder" />

        <div v-if="validationResult.status != 'none'" id="alert">
          <span class="material-symbols-outlined icon-button" id="alert-symbol">
            error
          </span>
          <p v-if="validationResult.message != ''" id="alert-message">
            {{ validationResult.message }}
          </p>
        </div>

        <div id="button-row">
          <button id="cancel-button" @click="$emit('cancelled')">Cancel</button>

          <button
            :class="{
              error: validationResult
                ? validationResult.status == 'error'
                : false,
            }"
            :disabled="
              validationResult ? validationResult.status == 'error' : false
            "
            @click="$emit('confirmed', inputValue)"
          >
            {{ buttonText }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, Transition, watch } from 'vue'

const props = defineProps([
  'text',
  'buttonText',
  'display',
  'default',
  'placeholder',
  'validation',
])
defineEmits(['confirmed', 'cancelled'])

const inputValue = ref('')

watch(
  () => props.display,
  display => {
    if (!display) return

    inputValue.value = ''

    if (props.default) {
      inputValue.value = props.default
    }
  }
)

const validationResult = computed(() =>
  props.validation ? props.validation(inputValue.value) : { status: 'none' }
)

onMounted(() => {
  inputValue.value = ''

  if (props.default) {
    inputValue.value = props.default
  }
})
</script>

<style scoped>
#component {
  position: absolute;

  top: 0;
  left: 0;

  width: 100%;
  height: 100%;

  background: var(--fade);

  z-index: 9999;

  display: flex;
  justify-content: center;
  align-items: center;

  backdrop-filter: blur(2px);

  filter: drop-shadow(0px 4px 4px rgba(0, 0, 0, 0.25));
}

#popup {
  background-color: var(--secondary);

  display: flex;
  flex-direction: column;

  align-items: center;

  width: max-content;
  max-width: 20rem;

  box-sizing: content-box;

  padding: 1rem;

  border-radius: 5px;
}

p {
  display: table;

  margin: 0;
  margin-bottom: 1rem;
}

#button-row {
  display: flex;
  gap: 0.5rem;

  margin-left: auto;
}

#cancel-button {
  background-color: var(--tertiary);
}

button {
  background-color: var(--grab);
  border-radius: 5px;

  padding: 0.25rem 0.5rem;

  transition: transform 200ms ease-in-out;
}

button:hover:not(.error) {
  transform: scale(1.05);
}

button.error {
  background-color: var(--alternate-grab);
  color: var(--alternate-text);
}

input {
  background: var(--tertiary);

  color: var(--text);

  margin-bottom: 1rem;

  padding: 0.25rem;

  font-size: small;

  width: 16rem;
}

::placeholder {
  color: var(--alternate-text);
}

#alert {
  display: flex;
  align-items: center;

  margin-top: -0.75rem;
  margin-bottom: 1rem;

  width: 100%;
}

#alert-message {
  color: var(--grab);

  font-size: small;

  margin: 0;
}

#alert-symbol {
  scale: 0.75;

  height: 1.5rem;

  color: var(--grab);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 100ms ease;
}
</style>

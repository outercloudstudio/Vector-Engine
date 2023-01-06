<template>
  <div id="component" :class="{ selected }">
    <div id="preview"></div>

    <p v-if="!renaming" id="name" @click="nameClicked">{{ name }}</p>
    <input
      v-if="renaming"
      ref="nameInput"
      id="name-input"
      @click="nameClicked"
      v-model.lazy="newNameBuffer"
      @blur="newNameBuffer = (<HTMLInputElement>$event.target)?.value"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, Ref, ref, watch } from 'vue'
import router from '@/router'

const newNameBuffer = computed({
  get: () => newName.value,
  set: value => {
    if (value != '') {
      newName.value = value
    } else {
      const cachedValue = newName.value

      newName.value = ''

      newName.value = cachedValue
    }
  },
})

const props = defineProps(['name', 'selected'])
const emit = defineEmits(['renamed'])

const renaming = ref(false)
const newName = ref('')
const nameInput: Ref<null | HTMLInputElement> = ref(null)

let lastNameClickTime: null | number = null

watch(
  () => props.selected,
  value => {
    if (value) return

    lastNameClickTime = null

    if (renaming.value && newName.value != props.name) {
      emit('renamed', props.name, newName.value)
    }

    renaming.value = false
  }
)

function nameClicked(event: Event) {
  if (lastNameClickTime == null) {
    lastNameClickTime = Date.now()
  } else {
    const now = Date.now()

    if (now - lastNameClickTime < 500) {
      lastNameClickTime = null

      renaming.value = true

      nextTick(() => {
        nameInput.value?.focus()
      })
    } else {
      lastNameClickTime = Date.now()
    }
  }
}

onMounted(() => {
  newName.value = props.name
})
</script>

<style scoped>
#component {
  margin: 0.5rem;
}

#preview {
  width: 9rem;
  height: 6rem;

  background-color: var(--secondary);

  border-radius: 5px;

  box-sizing: border-box;

  border: solid 0px rgba(0, 0, 0, 0);

  transition: border 100ms ease-in-out;
}

#component:hover #preview,
#component.selected #preview {
  border: solid 2px var(--grab);
}

#name {
  margin: 0;
  margin-top: 0.5rem;

  font-size: small;
  text-align: center;

  transition: color 100ms ease-in-out;

  color: var(--alternate-text);
}

#component:hover #name,
#component.selected #name {
  color: var(--grab);
}

#name-input {
  margin: 0;
  margin-top: 0.5rem;
  margin-left: auto;
  margin-right: auto;

  padding: 0;

  font-size: small;
  text-align: center;

  transition: color 100ms ease-in-out;

  color: var(--text);

  width: 8rem;

  background: none;

  display: block;
}
</style>

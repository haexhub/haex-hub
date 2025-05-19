<template>
  <form class="url-bar" @submit.prevent="handleSubmit">
    <input type="text" v-model="inputValue" placeholder="URL eingeben" />
    <span v-if="isLoading" class="loading-indicator">Laden...</span>
    <button v-else type="submit">Go</button>
  </form>
</template>

<script setup lang="ts">
const props = defineProps({
  url: {
    type: String,
    default: '',
  },
  isLoading: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['submit'])

const inputValue = ref(props.url)

watch(
  () => props.url,
  (newUrl) => {
    inputValue.value = newUrl
  }
)

const handleSubmit = () => {
  // URL validieren und ggf. Protokoll hinzufügen
  let processedUrl = inputValue.value.trim()
  if (processedUrl && !processedUrl.match(/^[a-zA-Z]+:\/\//)) {
    processedUrl = 'https://' + processedUrl
  }

  emit('submit', processedUrl)
}
</script>

<template>
  <div
    v-if="data"
    class="fixed top-2 right-2 bg-black/90 text-white text-xs p-3 rounded-lg shadow-2xl max-w-sm z-[9999] backdrop-blur-sm"
  >
    <div class="flex justify-between items-start gap-3 mb-2">
      <span class="font-bold text-sm">{{ title }}</span>
      <div class="flex gap-1">
        <button
          class="bg-white/20 hover:bg-white/30 px-2 py-1 rounded text-xs transition-colors"
          @click="copyToClipboardAsync"
        >
          Copy
        </button>
        <button
          v-if="dismissible"
          class="bg-white/20 hover:bg-white/30 px-2 py-1 rounded text-xs transition-colors"
          @click="handleDismiss"
        >
          ✕
        </button>
      </div>
    </div>
    <pre class="text-xs whitespace-pre-wrap font-mono overflow-auto max-h-96">{{ formattedData }}</pre>
  </div>
</template>

<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    data: Record<string, any> | null
    title?: string
    dismissible?: boolean
  }>(),
  {
    title: 'Debug Info',
    dismissible: false,
  },
)

const emit = defineEmits<{
  dismiss: []
}>()

const formattedData = computed(() => {
  if (!props.data) return ''
  return JSON.stringify(props.data, null, 2)
})

const copyToClipboardAsync = async () => {
  try {
    await navigator.clipboard.writeText(formattedData.value)
  } catch (err) {
    console.error('Failed to copy debug info:', err)
  }
}

const handleDismiss = () => {
  emit('dismiss')
}
</script>

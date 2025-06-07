<template>
  <div class="card">
    <slot name="image" />

    <div class="card-header">
      <slot name="header">
        <div
          v-if="$slots.title || title"
          class="flex items-center gap-2"
        >
          <Icon
            v-if="icon"
            :name="icon"
            size="28"
          />
          <h5
            v-if="title"
            class="card-title mb-0"
          >
            {{ title }}
          </h5>
          <slot
            v-else
            name="title"
          />
        </div>
        <div class="text-base-content/45">{{ subtitle }}</div>
      </slot>
    </div>

    <div class="card-body px-2 sm:px-6">
      <slot />
      <div
        v-if="$slots.action"
        class="card-actions"
      >
        <slot name="action" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const emit = defineEmits(['close', 'submit'])

defineProps<{ title?: string; subtitle?: string; icon?: string }>()

const { escape, enter } = useMagicKeys()

watchEffect(async () => {
  if (escape.value) {
    await nextTick()
    emit('close')
  }
})

watchEffect(async () => {
  if (enter.value) {
    await nextTick()
    emit('submit')
  }
})
</script>

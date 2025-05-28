<template>
  <UiDropdown :items="availableThemes" class="btn btn-primary btn-outline" @select="(theme) => $emit('select', theme)">
    <template #activator>
      <Icon :name="currentTheme.icon" />
    </template>

    <template #item="{ item }">
      <div class="flex gap-2 justify-center">
        <Icon :name="item.icon" class="my-auto" />
        <p>
          {{ item.name }}
        </p>
      </div>
    </template>
  </UiDropdown>
</template>

<script setup lang="ts">
const { availableThemes, currentTheme } = storeToRefs(useUiStore())

defineEmits<{ select: [ITheme] }>()
</script>
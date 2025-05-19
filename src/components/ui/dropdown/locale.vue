<template>
  <UiDropdown
    :items="availableLocales"
    @select="(locale) => $emit('select', locale)"
    activator-class="btn btn-primary"
  >
    <template #label>
      <Icon :name="flags[locale]" />
    </template>

    <template #item="{ item }">
      <div class="flex gap-2 justify-center">
        <Icon :name="flags[item]" class="my-auto" />
        <p>
          {{ item }}
        </p>
      </div>
    </template>
  </UiDropdown>
</template>

<script setup lang="ts">
import { type Locale } from 'vue-i18n'

const flags = {
  de: 'emojione:flag-for-germany',
  en: 'emojione:flag-for-united-kingdom',
}

const { availableLocales, locale } = useI18n()

defineEmits<{ select: [Locale] }>()
</script>

<template>
  <UiDropdown
    :items="availableLocales"
    class="btn btn-primary btn-outline"
    @select="(locale) => $emit('select', locale)"
  >
    <template #activator>
      <Icon :name="flags[locale]" />
      <Icon
        name="tabler:chevron-down"
        class="dropdown-open:rotate-180 size-4"
      />
    </template>

    <template #item="{ item }">
      <div class="flex gap-2 justify-center">
        <Icon
          :name="flags[item]"
          class="my-auto"
        />
        <p>
          {{ item }}
        </p>
      </div>
    </template>
  </UiDropdown>
</template>

<script setup lang="ts">
import type { Locale } from 'vue-i18n'

const flags = {
  de: 'emojione:flag-for-germany',
  en: 'emojione:flag-for-united-kingdom',
}

const { availableLocales, locale } = useI18n()

defineEmits<{ select: [Locale] }>()
</script>

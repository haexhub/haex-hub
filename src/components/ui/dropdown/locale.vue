<template>
  <UDropdownMenu
    arrow
    :items
    :ui="{}"
    :size="isSmallScreen ? 'lg' : 'md'"
  >
    <UButton
      :icon="items.find((item) => item.label === locale)?.icon"
      :label="locale"
      color="neutral"
      variant="outline"
    />
  </UDropdownMenu>
</template>

<script setup lang="ts">
import type { DropdownMenuItem } from '@nuxt/ui'
import type { Locale } from 'vue-i18n'

const { locales, locale } = useI18n()

const flags = {
  de: 'circle-flags:de',
  en: 'circle-flags:uk',
}

const emit = defineEmits<{ select: [Locale] }>()

const items = computed<DropdownMenuItem[]>(() =>
  locales.value.map((locale) => ({
    label: locale.code,
    icon: flags[locale.code],
    onSelect() {
      emit('select', locale.code)
    },
  })),
)

const { isSmallScreen } = storeToRefs(useUiStore())
</script>

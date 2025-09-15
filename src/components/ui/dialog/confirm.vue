<template>
  <UModal
    v-model:open="open"
    :title
    :description
    :fullscreen="isSmallScreen"
    :ui="{ header: 'pt-10 sm:pt-0', footer: 'mb-10 sm:mb-0' }"
  >
    <slot>
      <!-- <UiButton
        color="primary"
        variant="outline"
        icon="mdi:menu"
        :ui="{
          base: '',
        }"
      /> -->
    </slot>

    <template #title>
      <slot name="title" />
    </template>

    <template #body>
      <slot name="body" />
    </template>

    <template #footer>
      <div class="flex flex-col sm:flex-row gap-4 justify-end w-full">
        <UiButton
          :icon="abortIcon || 'mdi:close'"
          :label="abortLabel || t('abort')"
          block
          color="error"
          variant="outline"
          @click="open = false"
        />
        <UiButton
          :icon="confirmIcon || 'mdi:check'"
          :label="confirmLabel || t('confirm')"
          block
          color="primary"
          varaint="solid"
          @click="$emit('confirm')"
        />
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import { breakpointsTailwind, useBreakpoints } from '@vueuse/core'

defineProps<{
  abortIcon?: string
  abortLabel?: string
  confirmIcon?: string
  confirmLabel?: string
  description?: string
  title?: string
}>()

const open = defineModel<boolean>('open', { default: false })

const { t } = useI18n()
defineEmits(['confirm'])

const breakpoints = useBreakpoints(breakpointsTailwind)

// "smAndDown" gilt für sm, xs usw.
const isSmallScreen = breakpoints.smaller('sm')
</script>

<i18n lang="yaml">
de:
  abort: Abbrechen
  confirm: Bestätigen

en:
  abort: Abort
  confirm: Confirm
</i18n>

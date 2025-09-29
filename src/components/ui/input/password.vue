<template>
  <UiInput
    v-model="value"
    :label="t('label')"
    :leading-icon
    :placeholder="placeholder || ' '"
    :read-only
    :type="show ? 'text' : 'password'"
    :with-copy-button
  >
    <template #trailing>
      <UiButton
        aria-controls="password"
        color="neutral"
        variant="link"
        :aria-label="show ? t('hide') : t('show')"
        :aria-pressed="show"
        :icon="show ? 'i-lucide-eye-off' : 'i-lucide-eye'"
        :tooltip="show ? t('hide') : t('show')"
        size="sm"
        @click="show = !show"
      />
    </template>
  </UiInput>
</template>

<script setup lang="ts">
import type { AcceptableValue } from '@nuxt/ui/runtime/types/utils.js'

defineProps<{
  label?: string
  placeholder?: string
  leadingIcon?: string
  withCopyButton?: boolean
  readOnly?: boolean
}>()
const value = defineModel<AcceptableValue | undefined>()

const show = ref(false)
const { t } = useI18n()
</script>

<i18n lang="yaml">
de:
  show: Passwort ansehen
  hide: Passwort verstecken
  label: Passwort

en:
  show: Show password
  hide: Hide password
  label: Password
</i18n>

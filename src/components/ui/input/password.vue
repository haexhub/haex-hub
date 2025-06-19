<template>
  <UiInput
    v-model="value"
    :autofocus
    :check-input
    :label="label || t('password')"
    :placeholder="placeholder || t('password')"
    :rules
    :type="type"
    :with-copy-button
    @keyup="(e) => $emit('keyup', e)"
  >
    <template #append>
      <slot name="append" />

      <UiButton
        class="btn-outline btn-accent btn-square join-item"
        @click="tooglePasswordType"
      >
        <Icon :name="type === 'password' ? 'mdi:eye-off' : 'mdi:eye'" />
      </UiButton>
    </template>
  </UiInput>
</template>

<script setup lang="ts">
import type { ZodSchema } from 'zod'

const { t } = useI18n()

const value = defineModel<string | number | null | undefined>()

defineProps<{
  autofocus?: boolean
  checkInput?: boolean
  label?: string
  placeholder?: string
  rules?: ZodSchema
  withCopyButton?: boolean
}>()

defineEmits<{
  keyup: [KeyboardEvent]
}>()

const type = ref<'password' | 'text'>('password')

const tooglePasswordType = () => {
  type.value = type.value === 'password' ? 'text' : 'password'
}
</script>

<i18n lang="json">
{
  "de": {
    "password": "Passwort"
  },
  "en": {
    "password": "Password"
  }
}
</i18n>

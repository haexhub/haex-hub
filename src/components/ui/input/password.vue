<template>
  <UiInput
    :check-input
    :label="label || t('password')"
    :placeholder="placeholder || t('password')"
    :rules
    :type="type"
    :autofocus
    v-model="value"
  >
    <template #append>
      <UiButton
        class="btn-outline btn-accent btn-square h-auto"
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

defineProps({
  label: String,
  placeholder: String,
  checkInput: Boolean,
  rules: Object as PropType<ZodSchema>,
  autofocus: Boolean,
})

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

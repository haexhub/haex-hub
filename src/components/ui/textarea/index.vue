<template>
  <div class="relative">
    <UiButton
      v-if="withCopyButton"
      :tooltip="t('copy')"
      class="btn-square btn-outline btn-accent absolute z-10 top-2 right-2"
      @click="copy(`${value}`)"
    >
      <Icon :name="copied ? 'mdi:check' : 'mdi:content-copy'" />
    </UiButton>

    <div class="textarea-floating">
      <textarea
        :class="{ 'pr-10': withCopyButton }"
        :id
        :placeholder
        :readonly="read_only"
        class="textarea"
        v-bind="$attrs"
        v-model="value"
      ></textarea>
      <label
        class="textarea-floating-label"
        :for="id"
      >
        {{ label }}
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  placeholder?: string
  label?: string
  read_only?: boolean
  withCopyButton?: boolean
}>()

const id = useId()

const value = defineModel<string | null | undefined>()

const { copy, copied } = useClipboard()

const { t } = useI18n()
</script>

<i18n lang="yaml">
de:
  copy: Kopieren
en:
  copy: Copy
</i18n>

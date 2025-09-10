<template>
  <div>
    <UTextarea
      :id
      v-model="value"
      :ui="{ base: 'peer' }"
      :readonly="readOnly"
      class="w-full"
      v-bind="$attrs"
    >
      <label
        class="absolute pointer-events-none -top-2.5 left-0 text-highlighted text-xs font-medium px-1.5 transition-all peer-focus:-top-2.5 peer-focus:text-highlighted peer-focus:text-xs peer-focus:font-medium peer-placeholder-shown:text-sm peer-placeholder-shown:text-dimmed peer-placeholder-shown:top-1.5 peer-placeholder-shown:font-normal"
      >
        <span class="inline-flex bg-default px-1">
          {{ props.label }}
        </span>
      </label>

      <template #trailing>
        <UiButton
          v-show="withCopyButton"
          :color="copied ? 'success' : 'neutral'"
          :tooltip="t('copy')"
          :icon="copied ? 'mdi:check' : 'mdi:content-copy'"
          size="sm"
          variant="link"
          @click="copy(`${value}`)"
        />
      </template>
    </UTextarea>
  </div>
</template>

<script setup lang="ts">
import type { TextareaProps } from '@nuxt/ui'

interface ITextareaProps extends /* @vue-ignore */ TextareaProps {
  tooltip?: string
  withCopyButton?: boolean
  readOnly?: boolean
  label?: string
}

const props = defineProps<ITextareaProps>()

/* defineProps<{
  placeholder?: string
  label?: string
  readOnly?: boolean
  withCopyButton?: boolean
}>() */

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

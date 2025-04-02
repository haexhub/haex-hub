<template>
  <UiInput
    :autofocus
    :check-input="checkInput"
    :label="label || t('url')"
    :placeholder="placeholder || t('url')"
    :read_only
    :rules
    :with-copy-button
    v-model.trim="value"
  >
    <template #append>
      <UiButton
        v-if="read_only"
        @click="openUrl(`${value}`)"
        class="btn-outline btn-accent h-auto"
        :class="{
          disabled: !value?.length,
        }"
      >
        <Icon name="streamline:web" />
      </UiButton>
    </template>
  </UiInput>
</template>

<script setup lang="ts">
import type { ZodSchema } from 'zod';
import { openUrl } from '@tauri-apps/plugin-opener';

const { t } = useI18n();
const { currentScreenSize } = storeToRefs(useUiStore());

const value = defineModel<string | null | undefined>();

defineProps({
  label: String,
  placeholder: String,
  checkInput: Boolean,
  rules: Object as PropType<ZodSchema>,
  autofocus: Boolean,
  withCopyButton: Boolean,
  read_only: Boolean,
});
</script>

<i18n lang="json">
{
  "de": {
    "url": "Url"
  },
  "en": {
    "url": "Url"
  }
}
</i18n>

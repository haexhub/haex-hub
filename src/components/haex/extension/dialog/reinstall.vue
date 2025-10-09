<template>
  <UiDialogConfirm
    v-model:open="open"
    @abort="onDeny"
    @confirm="onConfirm"
  >
    <template #title>
      {{ t('title', { extensionName: preview?.manifest.name }) }}
    </template>

    <template #body>
      <div class="flex flex-col gap-4">
        <p>{{ t('question', { extensionName: preview?.manifest.name }) }}</p>

        <UAlert
          color="warning"
          variant="soft"
          :title="t('warning.title')"
          :description="t('warning.description')"
          icon="i-heroicons-exclamation-triangle"
        />

        <div
          v-if="preview"
          class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4"
        >
          <div class="flex items-center gap-3">
            <UIcon
              v-if="preview.manifest.icon"
              :name="preview.manifest.icon"
              class="w-12 h-12"
            />
            <div class="flex-1">
              <h4 class="font-semibold">
                {{ preview.manifest.name }}
              </h4>
              <p class="text-sm text-gray-500 dark:text-gray-400">
                {{ t('version') }}: {{ preview.manifest.version }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </template>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
import type { ExtensionPreview } from '~~/src-tauri/bindings/ExtensionPreview'

const { t } = useI18n()

const open = defineModel<boolean>('open', { default: false })
const preview = defineModel<ExtensionPreview | null>('preview', {
  default: null,
})

const emit = defineEmits(['deny', 'confirm'])

const onDeny = () => {
  open.value = false
  emit('deny')
}

const onConfirm = () => {
  open.value = false
  emit('confirm')
}
</script>

<i18n lang="yaml">
de:
  title: '{extensionName} bereits installiert'
  question: Soll die Erweiterung {extensionName} erneut installiert werden?
  warning:
    title: Achtung
    description: Die vorhandene Version wird vollständig entfernt und durch die neue Version ersetzt. Dieser Vorgang kann nicht rückgängig gemacht werden.
  version: Version

en:
  title: '{extensionName} is already installed'
  question: Do you want to reinstall {extensionName}?
  warning:
    title: Warning
    description: The existing version will be completely removed and replaced with the new version. This action cannot be undone.
  version: Version
</i18n>
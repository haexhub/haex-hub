<template>
  <UiDialogConfirm
    v-model:open="open"
    @abort="onAbort"
    @confirm="onConfirm"
  >
    <template #title>
      {{ t('title') }}
    </template>

    <template #body>
      <div class="flex flex-col gap-4">
        <i18n-t
          keypath="question"
          tag="p"
        >
          <template #name>
            <span class="font-bold text-primary">{{ extension?.name }}</span>
          </template>
        </i18n-t>

        <UAlert
          color="error"
          variant="soft"
          :title="t('warning.title')"
          :description="t('warning.description')"
          icon="i-heroicons-exclamation-triangle"
        />

        <div
          v-if="extension"
          class="bg-gray-100 dark:bg-gray-800 rounded-lg p-4"
        >
          <div class="flex items-center gap-3">
            <UIcon
              v-if="extension.icon"
              :name="extension.icon"
              class="w-12 h-12"
            />
            <UIcon
              v-else
              name="i-heroicons-puzzle-piece"
              class="w-12 h-12 text-gray-400"
            />
            <div class="flex-1">
              <h4 class="font-semibold">
                {{ extension.name }}
              </h4>
              <p class="text-sm text-gray-500 dark:text-gray-400">
                {{ t('version') }}: {{ extension.version }}
              </p>
              <p
                v-if="extension.author"
                class="text-sm text-gray-500 dark:text-gray-400"
              >
                {{ t('author') }}: {{ extension.author }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </template>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
import type { IHaexHubExtension } from '~/types/haexhub'

const emit = defineEmits(['confirm', 'abort'])

const { t } = useI18n()

defineProps<{ extension?: IHaexHubExtension }>()

const open = defineModel<boolean>('open')

const onAbort = () => {
  open.value = false
  emit('abort')
}

const onConfirm = () => {
  open.value = false
  emit('confirm')
}
</script>

<i18n lang="yaml">
de:
  title: Erweiterung entfernen
  question: Möchtest du {name} wirklich entfernen?
  warning:
    title: Achtung
    description: Diese Aktion kann nicht rückgängig gemacht werden. Alle Daten der Erweiterung werden dauerhaft gelöscht.
  version: Version
  author: Autor

en:
  title: Remove Extension
  question: Do you really want to remove {name}?
  warning:
    title: Warning
    description: This action cannot be undone. All extension data will be permanently deleted.
  version: Version
  author: Author
</i18n>

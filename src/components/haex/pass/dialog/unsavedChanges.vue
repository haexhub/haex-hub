<template>
  <UiDialogConfirm
    v-model:open="showUnsavedChangesDialog"
    :confirm-label="t('label')"
    :title="t('title')"
    @abort="$emit('abort')"
    @confirm="onConfirm"
  >
    <template #body>
      <div class="flex items-center h-full">
        {{ t('question') }}
      </div>
    </template>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
const { t } = useI18n()

const showUnsavedChangesDialog = defineModel<boolean>('open')
const ignoreChanges = defineModel<boolean>('ignoreChanges')
const { hasChanges } = defineProps<{ hasChanges: boolean }>()

const emit = defineEmits(['confirm', 'abort'])

const onConfirm = () => {
  ignoreChanges.value = true
  emit('confirm')
}

onBeforeRouteLeave(() => {
  if (hasChanges && !ignoreChanges.value) {
    showUnsavedChangesDialog.value = true
    return false
  }

  return true
})
</script>

<i18n lang="yaml">
de:
  title: Nicht gespeicherte Änderungen
  question: Sollen die Änderungen verworfen werden?
  label: Verwerfen

en:
  title: Unsaved changes
  question: Should the changes be discarded?
  label: Discard
</i18n>

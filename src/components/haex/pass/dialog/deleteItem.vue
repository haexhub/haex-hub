<template>
  <UiDialogConfirm
    v-model:open="showConfirmDeleteDialog"
    :confirm-label="final ? t('final.label') : t('label')"
    :title="final ? t('final.title') : t('title', { itemName })"
    @abort="$emit('abort')"
    @confirm="$emit('confirm')"
  >
    <template #body>
      {{
        final ? t('final.question', { itemName }) : t('question', { itemName })
      }}
    </template>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
const { t } = useI18n()

const showConfirmDeleteDialog = defineModel<boolean>('open')
defineProps<{ final?: boolean; itemName?: string | null }>()

defineEmits(['confirm', 'abort'])
</script>

<i18n lang="yaml">
de:
  title: Eintrag löschen
  question: Soll der Eintrag "{itemName}" in den Papierkorb verschoben werden?
  label: Verschieben

  final:
    title: Eintrag endgültig löschen
    question: Soll der Eintrag "{itemName}" endgültig gelöscht werden?
    label: Löschen

en:
  title: Delete Entry
  question: Should the “{itemName}” entry be moved to the recycle bin?
  label: Move

  final:
    title: Delete entry permanently
    question: Should the entry “{itemName}” be permanently deleted?
    label: Delete
</i18n>

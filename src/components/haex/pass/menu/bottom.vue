<template>
  <div
    class="fixed bottom-4 flex justify-between transition-all pointer-events-none right-0 sm:items-center items-end h-12"
    :class="[isVisible ? 'left-16' : 'left-0']"
  >
    <div class="flex items-center justify-center flex-1">
      <UiButton
        v-show="showCloseButton"
        :tooltip="t('abort')"
        icon="mdi:close"
        color="error"
        variant="ghost"
        class="pointer-events-auto"
        @click="$emit('close')"
      />
    </div>

    <div>
      <UiButton
        v-show="showEditButton"
        icon="mdi:pencil-outline"
        class="pointer-events-auto"
        size="xl"
        :tooltip="t('edit')"
        @click="$emit('edit')"
      />

      <UiButton
        v-show="showReadonlyButton"
        icon="mdi:pencil-off-outline"
        class="pointer-events-auto"
        size="xl"
        :tooltip="t('readonly')"
        @click="$emit('readonly')"
      />

      <UiButton
        v-show="showSaveButton"
        icon="mdi:content-save-outline"
        size="xl"
        class="pointer-events-auto"
        :class="{ 'animate-pulse': hasChanges }"
        :tooltip="t('save')"
        @click="$emit('save')"
      />
    </div>

    <div class="flex items-center justify-center flex-1">
      <UiButton
        v-show="showDeleteButton"
        color="error"
        icon="mdi:trash-outline"
        class="pointer-events-auto"
        variant="ghost"
        :tooltip="t('delete')"
        @click="$emit('delete')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
const { isVisible } = storeToRefs(useSidebarStore())
const { t } = useI18n()

defineProps<{
  hasChanges?: boolean
  showCloseButton?: boolean
  showDeleteButton?: boolean
  showEditButton?: boolean
  showReadonlyButton?: boolean
  showSaveButton?: boolean
}>()

defineEmits(['close', 'edit', 'readonly', 'save', 'delete'])
</script>

<i18n lang="yaml">
de:
  save: Speichern
  abort: Abbrechen
  edit: Bearbeiten
  readonly: Lesemodus
  delete: Löschen

en:
  save: Save
  abort: Abort
  edit: Edit
  readonly: Read Mode
  delete: Delete
</i18n>

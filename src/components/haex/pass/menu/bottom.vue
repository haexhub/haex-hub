<template>
  <div
    class="fixed bottom-4 flex justify-between transition-all pointer-events-none right-0 sm:items-center items-end"
    :class="[isVisible ? 'left-15 ' : 'left-0']"
  >
    <div class="flex items-center justify-center flex-1">
      <UiButton
        v-show="showCloseButton"
        :tooltip="t('abort')"
        @click="$emit('close')"
        class="btn-accent btn-square"
      >
        <Icon name="mdi:close" />
      </UiButton>
    </div>

    <div>
      <UiButton
        v-show="showEditButton"
        :tooltip="t('edit')"
        @click="$emit('edit')"
        class="btn-xl btn-square btn-primary"
      >
        <Icon
          name="mdi:pencil-outline"
          class="size-11 shrink-0"
        />
      </UiButton>

      <UiButton
        v-show="showReadonlyButton"
        :tooltip="t('readonly')"
        class="btn-xl btn-square btn-primary"
        @click="$emit('readonly')"
      >
        <Icon
          name="mdi:pencil-off-outline"
          class="size-11 shrink-0"
        />
      </UiButton>

      <UiButton
        v-show="showSaveButton"
        :tooltip="t('save')"
        class="btn-xl btn-square btn-primary motion-duration-2000"
        :class="{ 'motion-preset-pulse-sm': hasChanges }"
        @click="$emit('save')"
      >
        <Icon
          name="mdi:content-save-outline"
          class="size-11 shrink-0"
        />
      </UiButton>
    </div>

    <div class="flex items-center justify-center flex-1">
      <UiButton
        v-show="showDeleteButton"
        :tooltip="t('delete')"
        class="btn-square btn-error"
        @click="$emit('delete')"
      >
        <Icon
          name="mdi:trash-outline"
          class="shrink-0"
        />
      </UiButton>
    </div>
  </div>
</template>

<script setup lang="ts">
const { isVisible } = storeToRefs(useSidebarStore())
const { t } = useI18n()

defineProps<{
  showCloseButton?: boolean
  showDeleteButton?: boolean
  showEditButton?: boolean
  showReadonlyButton?: boolean
  showSaveButton?: boolean
  hasChanges?: boolean
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

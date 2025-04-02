<template>
  <UiDialog
    v-model:open="showConfirmation"
    :title="t('dialog.title')"
  >
    {{ t('dialog.question') }}
    <template #buttons>
      <UiButton
        class="btn-outline btn-error focus:bg-primary"
        tabindex="10"
        @click="$emit('reject')"
      >
        <Icon name="mdi:cancel" />
        <span class="hidden sm:block">
          {{ t('dialog.reject') }}
        </span>
      </UiButton>
      <UiButton
        class="btn-outline focus:bg-primary"
        tabindex="11"
        ref="abortButtonRef"
        @click="showConfirmation = false"
      >
        <Icon name="mdi:close" />
        <span class="hidden sm:block">
          {{ t('dialog.abort') }}
        </span>
      </UiButton>
      <UiButton
        class="btn-outline btn-success"
        tabindex="12"
        @click="$emit('submit')"
      >
        <Icon name="mdi:check" />
        <span class="hidden sm:block">
          {{ t('dialog.save') }}
        </span>
      </UiButton>
    </template>
  </UiDialog>
</template>

<script setup lang="ts">
const showConfirmation = defineModel<boolean>();
const abortButtonRef = useTemplateRef('abortButtonRef');

const { t } = useI18n();
const { currentScreenSize } = storeToRefs(useUiStore());

onUpdated(() => {
  abortButtonRef.value?.$el.focus();
});

defineEmits(['submit', 'reject']);
</script>

<i18n lang="json">
{
  "de": {
    "dialog": {
      "title": "Ungespeicherte Änderungen",
      "question": "Möchten Sie die Änderungen speichern?",
      "reject": "Verwerfen",
      "abort": "Abbrechen",
      "save": "Speichern"
    }
  },
  "en": {
    "dialog": {
      "title": "Unsaved Changes",
      "question": "Would you like to save the changes?",
      "reject": "Reject",
      "abort": "Abort",
      "save": "Save"
    }
  }
}
</i18n>

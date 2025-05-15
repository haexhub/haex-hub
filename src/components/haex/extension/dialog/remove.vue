<template>
  <UiDialog :title="t('title')" v-model:open="open">
    <div>
      <i18n-t keypath="question" tag="p">
        <template #name>
          <span class="font-bold text-primary">{{ extension?.name }}</span>
        </template>
      </i18n-t>
    </div>
    <template #buttons>
      <UiButton class="btn-outline btn-error" @click="open = false">
        <Icon name="mdi:cancel" /> {{ t("abort") }}
      </UiButton>

      <UiButton class="btn-error" @click="onConfirm">
        <Icon name="mdi:trash" /> {{ t("remove") }}
      </UiButton>
    </template>
  </UiDialog>
</template>

<script setup lang="ts">
import type { IHaexHubExtension } from "~/types/haexhub";

const emit = defineEmits(["confirm"]);

const { t } = useI18n();

defineProps<{ extension?: IHaexHubExtension }>();

const open = defineModel<boolean>("open");

const onConfirm = () => {
  open.value = false;
  emit("confirm");
};
</script>

<i18n lang="json">
{
  "de": {
    "title": "Erweiterung löschen",
    "question": "Soll {name} wirklich gelöscht werden?",
    "abort": "Abbrechen",
    "remove": "Löschen"
  },
  "en": {
    "title": "Remove Extension",
    "question": "Soll {name} wirklich gelöscht werden?",
    "abort": "Abort",
    "remove": "Remove"
  }
}
</i18n>

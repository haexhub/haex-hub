<template>
  <UiDialogConfirm v-model:open="open">
    <template #title>
      <i18n-t keypath="title" tag="p">
        <template #extensionName>
          <span class="font-bold text-primary">{{ manifest?.name }}</span>
        </template>
      </i18n-t>
    </template>

    <p>{{ t("question", { extensionName: manifest?.name }) }}</p>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
import type { IHaexHubExtensionManifest } from "~/types/haexhub";

const { t } = useI18n();

const open = defineModel<boolean>("open", { default: false });
const { manifest } = defineProps<{ manifest?: IHaexHubExtensionManifest | null }>();

</script>

<i18n lang="yaml">
  de:
    title: "{extensionName} bereits installiert"
    question: Soll die Erweiterung {extensionName} erneut installiert werden?

  en:
    title: "{extensionName} is already installed"
    question: Do you want to reinstall {extensionName}?
</i18n>
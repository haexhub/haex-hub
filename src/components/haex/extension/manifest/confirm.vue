<template>
  <UiDialog :title="t('title')" v-model:open="open">
    <div>
      <i18n-t keypath="question" tag="p">
        <template #extension>
          <span class="font-bold text-primary">{{ manifest?.name }}</span>
        </template>
      </i18n-t>

      <!--  {{ t("question", { extension: manifest?.name }) }}
      <span class="font-bold text-primary">{{ manifest?.name }}</span> zu HaexHub hinzufügen? -->
    </div>
    <div class="flex flex-col">
      <HaexExtensionManifestPermissionsFilesystem
        v-if="manifest?.permissions?.filesystem"
        :filesystem="manifest?.permissions?.filesystem"
      />

      <HaexExtensionManifestPermissionsDatabase
        v-if="manifest?.permissions?.database"
        :database="manifest?.permissions?.database"
      />

      <HaexExtensionManifestPermissionsHttp
        v-if="manifest?.permissions?.http"
        :http="manifest?.permissions?.http"
      />
      <!-- <VaultCard>
        <template #header>
          <h3>{{ t("filesystem.title") }}</h3>
        </template>

        <div>
          {{ manifest?.permissions.filesystem }}
        </div>
      </VaultCard>

      <VaultCard>
        <template #header>
          <h3>{{ t("http.title") }}</h3>
        </template>

        <div>
          {{ manifest?.permissions.http }}
        </div>
      </VaultCard> -->
    </div>

    <template #buttons>
      <UiButton @click="onDeny" class="btn-error btn-outline">{{ t("deny") }} </UiButton>
      <UiButton @click="onConfirm" class="btn-success btn-outline">{{ t("confirm") }}</UiButton>
    </template>
  </UiDialog>
</template>

<script setup lang="ts">
import type { IHaexHubExtensionManifest } from "~/types/haexhub";

const { t } = useI18n();

const open = defineModel<boolean>("open", { default: false });
defineProps<{ manifest?: IHaexHubExtensionManifest | null }>();

const emit = defineEmits(["deny", "confirm"]);

const onDeny = () => {
  open.value = false;
  console.log("onDeny open", open.value);
  emit("deny");
};

const onConfirm = () => {
  open.value = false;
  console.log("onConfirm open", open.value);
  emit("confirm");
};
</script>

<i18n lang="json">
{
  "de": {
    "title": "Erweiterung hinzufügen",
    "question": "Möchtest du die Erweiterung {extension} hinzufügen?",
    "confirm": "Bestätigen",
    "deny": "Ablehnen",

    "permission": {
      "read": "Lesen",
      "write": "Schreiben"
    },

    "database": {
      "title": "Datenbank Berechtigungen"
    },
    "http": {
      "title": "Internet Berechtigungen"
    },
    "filesystem": {
      "title": "Dateisystem Berechtigungen"
    }
  },
  "en": { "title": "Confirm Permission" }
}
</i18n>

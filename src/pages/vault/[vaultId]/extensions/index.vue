<template>
  <div class="flex flex-col">
    <h1>{{ t("title") }}</h1>
    <UiButton @click="loadExtensionManifestAsync">
      {{ t("extension.add") }}
    </UiButton>

    <HaexExtensionManifestConfirm
      :manifest="extension.manifest!"
      v-model:open="showConfirmation"
      @confirm="addExtensionAsync"
    />
  </div>
</template>

<script setup lang="ts">
import { join } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";

definePageMeta({
  name: "extensionOverview",
});

const { t } = useI18n();
const extensionStore = useExtensionsStore();

const showConfirmation = ref(false);

const extension = reactive<{
  manifest: IHaexHubExtensionManifest | null | undefined;
  path: string | null;
}>({
  manifest: null,
  path: "",
});

const loadExtensionManifestAsync = async () => {
  try {
    extension.path = await open({ directory: true, recursive: true });
    if (!extension.path) return;

    const manifestFile = JSON.parse(
      await readTextFile(await join(extension.path, "manifest.json"))
    );

    if (!extensionStore.checkManifest(manifestFile))
      throw new Error(`Manifest fehlerhaft ${JSON.stringify(manifestFile)}`);

    extension.manifest = manifestFile;
    showConfirmation.value = true;
  } catch (error) {
    console.error("Fehler beim Laden des Moduls:", error);
  }
};

const { add } = useSnackbar();

const addExtensionAsync = async () => {
  try {
    await extensionStore.installAsync(extension.path);
    await extensionStore.loadExtensionsAsync();
    console.log("Modul erfolgreich geladen");
    add({
      type: "success",
      title: t("extension.success.title", { extension: extension.manifest?.name }),
      text: t("extension.success.text"),
    });
  } catch (error) {
    console.error("Fehler beim Laden des Moduls:", error);
    add({ type: "error", text: JSON.stringify(error) });
  }
};
</script>

<i18n lang="json">
{
  "de": {
    "title": "Erweiterung installieren",
    "extension": {
      "add": "Erweiterung hinzufügen",
      "success": {
        "title": "{extension} hinzugefügt",
        "text": "Die Erweiterung wurde erfolgreich hinzugefügt"
      }
    }
  },
  "en": {
    "title": "Install extension"
  }
}
</i18n>

<template>
  <div class="flex flex-col p-1 relative">
    <UiButton
      class="fixed top-20 right-4 btn-square btn-primary"
      @click="loadExtensionManifestAsync"
    >
      <Icon name="mdi:plus" size="1.5em" />
    </UiButton>
    <h1>{{ t("title") }}</h1>

    <div class="flex">
      <HaexExtensionCard
        v-for="extension in extensionStore.availableExtensions"
        v-bind="extension"
        @remove="onShowRemoveDialog(extension)"
      >
      </HaexExtensionCard>
    </div>
    <!-- <UiButton @click="loadExtensionManifestAsync">
      {{ t("extension.add") }}
    </UiButton> -->

    <HaexExtensionManifestConfirm
      :manifest="extension.manifest"
      v-model:open="showConfirmation"
      @confirm="addExtensionAsync"
    />

    {{ showRemoveDialog }}
    <HaexExtensionDialogRemove
      v-model:open="showRemoveDialog"
      :extension="extensionToBeRemoved"
      @confirm="removeExtensionAsync"
    >
    </HaexExtensionDialogRemove>
  </div>
</template>

<script setup lang="ts">
import { join } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";
import type { IHaexHubExtension, IHaexHubExtensionManifest } from "~/types/haexhub";

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

onMounted(() => console.log("extension overview"));

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
    console.error("Fehler loadExtensionManifestAsync:", error);
    add({ type: "error", text: JSON.stringify(error) });
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
    console.error("Fehler addExtensionAsync:", error);
    add({ type: "error", text: JSON.stringify(error) });
  }
};

const showRemoveDialog = ref(false);
const extensionToBeRemoved = ref<IHaexHubExtension>();

const onShowRemoveDialog = (extension: IHaexHubExtension) => {
  extensionToBeRemoved.value = extension;
  showRemoveDialog.value = true;
};

const removeExtensionAsync = async () => {
  if (!extensionToBeRemoved.value?.id || !extensionToBeRemoved.value?.version) {
    add({ type: "error", text: "Erweiterung kann nicht gelöscht werden" });
    return;
  }

  try {
    await extensionStore.removeExtensionAsync(
      extensionToBeRemoved.value.id,
      extensionToBeRemoved.value.version
    );
    await extensionStore.loadExtensionsAsync();
    add({
      type: "success",
      title: t("extension.remove.success.title", {
        extensionName: extensionToBeRemoved.value.name,
      }),
      text: t("extension.remove.success.text", { extensionName: extensionToBeRemoved.value.name }),
    });
  } catch (error) {
    add({
      type: "error",
      title: t("extension.remove.error.title"),
      text: t("extension.remove.error.text", { error: JSON.stringify(error) }),
    });
  }
};
</script>

<i18n lang="yaml">
de:
  title: "Erweiterung installieren"
  extension:
    remove:
      success:
        text: "Erweiterung {extensionName} wurde erfolgreich entfernt"
        title: "{extensionName} entfernt"
      error:
        text: "Erweiterung {extensionName} konnte nicht entfernt werden. \n {error}"
        title: "Fehler beim Entfernen von {extensionName}"

    add: "Erweiterung hinzufügen"
    success:
      title: "{extension} hinzugefügt"
      text: "Die Erweiterung wurde erfolgreich hinzugefügt"
en:
  title: "Install extension"
</i18n>

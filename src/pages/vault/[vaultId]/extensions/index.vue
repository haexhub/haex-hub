<template>
  <div class="flex flex-col p-4 relative h-full">
    <!-- <div
      v-if="extensionStore.availableExtensions.length"
      class="flex"
    >
      <UiButton
        class="fixed top-20 right-4"
        @click="onSelectExtensionAsync"
      >
        <Icon
          name="mdi:plus"
          size="1.5em"
        />
      </UiButton>

      <HaexExtensionCard
        v-for="_extension in extensionStore.availableExtensions"
        v-bind="_extension"
        :key="_extension.id"
        @remove="onShowRemoveDialog(_extension)"
      />
    </div> -->

    {{ preview }}
    <div class="h-full w-full">
      <div class="fixed top-30 right-10">
        <UiButton
          :tooltip="t('extension.add')"
          @click="onSelectExtensionAsync"
          square
          size="xl"
        >
          <Icon
            name="mdi:plus"
            size="1.5em"
          />
        </UiButton>
      </div>
    </div>

    <HaexExtensionDialogReinstall
      v-model:open="openOverwriteDialog"
      :manifest="extension.manifest"
      @confirm="addExtensionAsync"
    />

    <HaexExtensionDialogInstall
      v-model:open="showConfirmation"
      :preview="preview"
      @confirm="addExtensionAsync"
    />

    <HaexExtensionDialogRemove
      v-model:open="showRemoveDialog"
      :extension="extensionToBeRemoved"
      @confirm="removeExtensionAsync"
    />
  </div>
</template>

<script setup lang="ts">
import type {
  IHaexHubExtension,
  IHaexHubExtensionManifest,
} from '~/types/haexhub'
import { open } from '@tauri-apps/plugin-dialog'
import type { ExtensionPreview } from '~~/src-tauri/bindings/ExtensionPreview'

definePageMeta({
  name: 'extensionOverview',
})

const { t } = useI18n()
const extensionStore = useExtensionsStore()

const showConfirmation = ref(false)
const openOverwriteDialog = ref(false)

const extension = reactive<{
  manifest: IHaexHubExtensionManifest | null | undefined
  path: string | null
}>({
  manifest: null,
  path: '',
})

/* const loadExtensionManifestAsync = async () => {
  try {
    extension.path = await open({ directory: true, recursive: true })
    if (!extension.path) return

    const manifestFile = JSON.parse(
      await readTextFile(await join(extension.path, 'manifest.json')),
    )

    if (!extensionStore.checkManifest(manifestFile))
      throw new Error(`Manifest fehlerhaft ${JSON.stringify(manifestFile)}`)

    return manifestFile
  } catch (error) {
    console.error('Fehler loadExtensionManifestAsync:', error)
    add({ color: 'error', description: JSON.stringify(error) })
    await addNotificationAsync({ text: JSON.stringify(error), type: 'error' })
  }
} */

const { add } = useToast()
const { addNotificationAsync } = useNotificationStore()

const preview = ref<ExtensionPreview>()

const onSelectExtensionAsync = async () => {
  try {
    extension.path = await open({ directory: false, recursive: true })
    if (!extension.path) return

    preview.value = await extensionStore.previewManifestAsync(extension.path)

    if (!preview.value) return

    // Check if already installed
    const isAlreadyInstalled = await extensionStore.isExtensionInstalledAsync({
      id: preview.value.manifest.id,
      version: preview.value.manifest.version,
    })

    if (isAlreadyInstalled) {
      openOverwriteDialog.value = true
    } else {
      showConfirmation.value = true
    }
  } catch (error) {
    add({ color: 'error', description: JSON.stringify(error) })
    await addNotificationAsync({ text: JSON.stringify(error), type: 'error' })
  }
}

const addExtensionAsync = async () => {
  try {
    await extensionStore.installAsync(extension.path)
    await extensionStore.loadExtensionsAsync()

    add({
      color: 'success',
      title: t('extension.success.title', {
        extension: extension.manifest?.name,
      }),
      description: t('extension.success.text'),
    })
    await addNotificationAsync({
      text: t('extension.success.text'),
      type: 'success',
      title: t('extension.success.title', {
        extension: extension.manifest?.name,
      }),
    })
  } catch (error) {
    console.error('Fehler addExtensionAsync:', error)
    add({ color: 'error', description: JSON.stringify(error) })
    await addNotificationAsync({ text: JSON.stringify(error), type: 'error' })
  }
}

const showRemoveDialog = ref(false)
const extensionToBeRemoved = ref<IHaexHubExtension>()

const onShowRemoveDialog = (extension: IHaexHubExtension) => {
  extensionToBeRemoved.value = extension
  showRemoveDialog.value = true
}

const removeExtensionAsync = async () => {
  if (!extensionToBeRemoved.value?.id || !extensionToBeRemoved.value?.version) {
    add({
      color: 'error',
      description: 'Erweiterung kann nicht gelöscht werden',
    })
    return
  }

  try {
    await extensionStore.removeExtensionAsync(
      extensionToBeRemoved.value.id,
      extensionToBeRemoved.value.version,
    )
    await extensionStore.loadExtensionsAsync()
    add({
      color: 'success',
      title: t('extension.remove.success.title', {
        extensionName: extensionToBeRemoved.value.name,
      }),
      description: t('extension.remove.success.text', {
        extensionName: extensionToBeRemoved.value.name,
      }),
    })
    await addNotificationAsync({
      text: t('extension.remove.success.text', {
        extensionName: extensionToBeRemoved.value.name,
      }),
      type: 'success',
      title: t('extension.remove.success.title', {
        extensionName: extensionToBeRemoved.value.name,
      }),
    })
  } catch (error) {
    add({
      color: 'error',
      title: t('extension.remove.error.title'),
      description: t('extension.remove.error.text', {
        error: JSON.stringify(error),
      }),
    })
    await addNotificationAsync({
      type: 'error',
      title: t('extension.remove.error.title'),
      text: t('extension.remove.error.text', { error: JSON.stringify(error) }),
    })
  }
}
</script>

<i18n lang="yaml">
de:
  title: 'Erweiterung installieren'
  extension:
    remove:
      success:
        text: 'Erweiterung {extensionName} wurde erfolgreich entfernt'
        title: '{extensionName} entfernt'
      error:
        text: "Erweiterung {extensionName} konnte nicht entfernt werden. \n {error}"
        title: 'Fehler beim Entfernen von {extensionName}'

    add: 'Erweiterung hinzufügen'
    success:
      title: '{extension} hinzugefügt'
      text: 'Die Erweiterung wurde erfolgreich hinzugefügt'
en:
  title: 'Install extension'
  extension:
    remove:
      success:
        text: 'Extension {extensionName} was removed'
        title: '{extensionName} removed'
      error:
        text: "Extension {extensionName} couldn't be removed. \n {error}"
        title: 'Exception during uninstall {extensionName}'

    add: 'Add Extension'
    success:
      title: '{extension} added'
      text: 'Extensions was added successfully'
</i18n>

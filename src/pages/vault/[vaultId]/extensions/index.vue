<template>
  <div class="flex flex-col p-1 relative h-full">
    <div class="flex" v-if="extensionStore.availableExtensions.length">
      <UiButton
        class="fixed top-20 right-4 btn-square btn-primary"
        @click="loadExtensionManifestAsync"
      >
        <Icon name="mdi:plus" size="1.5em" />
      </UiButton>

      <HaexExtensionCard
        v-for="extension in extensionStore.availableExtensions"
        v-bind="extension"
        @remove="onShowRemoveDialog(extension)"
      >
      </HaexExtensionCard>
    </div>

    <!-- <SvgoExtensionsOverview class="h-screen w-screen" /> -->
    <!-- <nuxt-icon name="extensions-overview" class="size-full" /> -->
    <div v-else class="h-full w-full">
      <Icon
        name="my-icon:extensions-overview"
        class="size-full md:size-2/3 md:translate-x-1/5 md:translate-y-1/3"
      />
      <div class="fixed top-30 right-10">
        <UiTooltip :tooltip="t('extension.add')">
          <UiButton
            class="btn-square btn-primary btn-xl btn-gradient rotate-45"
            @click="loadExtensionManifestAsync"
          >
            <Icon name="mdi:plus" size="1.5em" class="rotate-45" />
          </UiButton>
        </UiTooltip>
      </div>
    </div>

    <HaexExtensionManifestConfirm
      :manifest="extension.manifest"
      v-model:open="showConfirmation"
      @confirm="addExtensionAsync"
    />

    <HaexExtensionDialogRemove
      v-model:open="showRemoveDialog"
      :extension="extensionToBeRemoved"
      @confirm="removeExtensionAsync"
    >
    </HaexExtensionDialogRemove>
  </div>
</template>

<script setup lang="ts">
import { join } from '@tauri-apps/api/path'
import { open } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'
import type {
  IHaexHubExtension,
  IHaexHubExtensionManifest,
} from '~/types/haexhub'

definePageMeta({
  name: 'extensionOverview',
})

const { t } = useI18n()
const extensionStore = useExtensionsStore()

const showConfirmation = ref(false)

const extension = reactive<{
  manifest: IHaexHubExtensionManifest | null | undefined
  path: string | null
}>({
  manifest: null,
  path: '',
})

const loadExtensionManifestAsync = async () => {
  try {
    extension.path = await open({ directory: true, recursive: true })
    if (!extension.path) return

    const manifestFile = JSON.parse(
      await readTextFile(await join(extension.path, 'manifest.json'))
    )

    if (!extensionStore.checkManifest(manifestFile))
      throw new Error(`Manifest fehlerhaft ${JSON.stringify(manifestFile)}`)

    extension.manifest = manifestFile
    showConfirmation.value = true
  } catch (error) {
    console.error('Fehler loadExtensionManifestAsync:', error)
    add({ type: 'error', text: JSON.stringify(error) })
  }
}

const { add } = useSnackbar()

const addExtensionAsync = async () => {
  try {
    await extensionStore.installAsync(extension.path)
    await extensionStore.loadExtensionsAsync()

    add({
      type: 'success',
      title: t('extension.success.title', {
        extension: extension.manifest?.name,
      }),
      text: t('extension.success.text'),
    })
  } catch (error) {
    console.error('Fehler addExtensionAsync:', error)
    add({ type: 'error', text: JSON.stringify(error) })
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
    add({ type: 'error', text: 'Erweiterung kann nicht gelöscht werden' })
    return
  }

  try {
    await extensionStore.removeExtensionAsync(
      extensionToBeRemoved.value.id,
      extensionToBeRemoved.value.version
    )
    await extensionStore.loadExtensionsAsync()
    add({
      type: 'success',
      title: t('extension.remove.success.title', {
        extensionName: extensionToBeRemoved.value.name,
      }),
      text: t('extension.remove.success.text', {
        extensionName: extensionToBeRemoved.value.name,
      }),
    })
  } catch (error) {
    add({
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
</i18n>

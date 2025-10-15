<template>
  <div class="p-4 max-w-4xl mx-auto space-y-6">
    <div class="space-y-2">
      <h1 class="text-2xl font-bold">{{ t('title') }}</h1>
      <p class="text-sm opacity-70">{{ t('description') }}</p>
    </div>

    <!-- Add Dev Extension Form -->
    <UCard class="p-4 space-y-4">
      <h2 class="text-lg font-semibold">{{ t('add.title') }}</h2>

      <div class="space-y-2">
        <label class="text-sm font-medium">{{ t('add.extensionPath') }}</label>
        <div class="flex gap-2">
          <UiInput
            v-model="extensionPath"
            :placeholder="t('add.extensionPathPlaceholder')"
            class="flex-1"
          />
          <UiButton
            :label="t('add.browse')"
            variant="outline"
            @click="browseExtensionPathAsync"
          />
        </div>
        <p class="text-xs opacity-60">{{ t('add.extensionPathHint') }}</p>
      </div>

      <UiButton
        :label="t('add.loadExtension')"
        :loading="isLoading"
        :disabled="!extensionPath"
        @click="loadDevExtensionAsync"
      />
    </UCard>

    <!-- List of Dev Extensions -->
    <div
      v-if="devExtensions.length > 0"
      class="space-y-2"
    >
      <h2 class="text-lg font-semibold">{{ t('list.title') }}</h2>

      <UCard
        v-for="ext in devExtensions"
        :key="ext.id"
        class="p-4 flex items-center justify-between"
      >
        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <h3 class="font-medium">{{ ext.name }}</h3>
            <UBadge color="info">DEV</UBadge>
          </div>
          <p class="text-sm opacity-70">v{{ ext.version }}</p>
          <p class="text-xs opacity-50">{{ ext.publicKey.slice(0, 16) }}...</p>
        </div>

        <div class="flex gap-2">
          <UiButton
            :label="t('list.reload')"
            variant="outline"
            size="sm"
            @click="reloadDevExtensionAsync(ext)"
          />
          <UiButton
            :label="t('list.remove')"
            variant="ghost"
            size="sm"
            color="error"
            @click="removeDevExtensionAsync(ext)"
          />
        </div>
      </UCard>
    </div>

    <div
      v-else
      class="text-center py-8 opacity-50"
    >
      {{ t('list.empty') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

definePageMeta({
  name: 'settings-developer',
})

const { t } = useI18n()
const { add } = useToast()
const { loadExtensionsAsync } = useExtensionsStore()


// State
const extensionPath = ref('')
const isLoading = ref(false)
const devExtensions = ref<
  Array<{
    id: string
    publicKey: string
    name: string
    version: string
    enabled: boolean
  }>
>([])

// Load dev extensions on mount
onMounted(async () => {
  await loadDevExtensionListAsync()
})

// Browse for extension directory
const browseExtensionPathAsync = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('add.browseTitle'),
    })

    if (selected && typeof selected === 'string') {
      extensionPath.value = selected
    }
  } catch (error) {
    console.error('Failed to browse directory:', error)
    add({
      description: t('add.errors.browseFailed'),
      color: 'error',
    })
  }
}

// Load a dev extension
const loadDevExtensionAsync = async () => {
  if (!extensionPath.value) return

  isLoading.value = true
  try {
    const extensionId = await invoke<string>('load_dev_extension', {
      extensionPath: extensionPath.value,
    })

    add({
      description: t('add.success'),
      color: 'success',
    })

    // Reload list
    await loadDevExtensionListAsync()

    // Reload all extensions in the main extension store so they appear in the launcher
    await loadExtensionsAsync()

    // Clear input
    extensionPath.value = ''
  } catch (error: any) {
    console.error('Failed to load dev extension:', error)
    add({
      description: error || t('add.errors.loadFailed'),
      color: 'error',
    })
  } finally {
    isLoading.value = false
  }
}

// Load all dev extensions (for the list on this page)
const loadDevExtensionListAsync = async () => {
  try {
    const extensions = await invoke<Array<any>>('get_all_dev_extensions')
    devExtensions.value = extensions
  } catch (error) {
    console.error('Failed to load dev extensions:', error)
  }
}

// Reload a dev extension (removes and re-adds)
const reloadDevExtensionAsync = async (ext: any) => {
  try {
    // Get the extension path from somewhere (we need to store this)
    // For now, just show a message
    add({
      description: t('list.reloadInfo'),
      color: 'info',
    })
  } catch (error: any) {
    console.error('Failed to reload dev extension:', error)
    add({
      description: error || t('list.errors.reloadFailed'),
      color: 'error',
    })
  }
}

// Remove a dev extension
const removeDevExtensionAsync = async (ext: any) => {
  try {
    await invoke('remove_dev_extension', {
      publicKey: ext.publicKey,
      name: ext.name,
    })

    add({
      description: t('list.removeSuccess'),
      color: 'success',
    })

    // Reload list
    await loadDevExtensionListAsync()

    // Reload all extensions store
    await loadExtensionsAsync()
  } catch (error: any) {
    console.error('Failed to remove dev extension:', error)
    add({
      description: error || t('list.errors.removeFailed'),
      color: 'error',
    })
  }
}
</script>

<i18n lang="yaml">
de:
  title: Entwicklereinstellungen
  description: Lade Extensions im Entwicklungsmodus für schnelleres Testen mit Hot-Reload.
  add:
    title: Dev-Extension hinzufügen
    extensionPath: Extension-Pfad
    extensionPathPlaceholder: /pfad/zu/deiner/extension
    extensionPathHint: Pfad zum Extension-Projekt (enthält haextension/ und haextension.json)
    browse: Durchsuchen
    browseTitle: Extension-Verzeichnis auswählen
    loadExtension: Extension laden
    success: Dev-Extension erfolgreich geladen
    errors:
      browseFailed: Verzeichnis konnte nicht ausgewählt werden
      loadFailed: Extension konnte nicht geladen werden
  list:
    title: Geladene Dev-Extensions
    empty: Keine Dev-Extensions geladen
    reload: Neu laden
    remove: Entfernen
    reloadInfo: Extension wird beim nächsten Laden automatisch aktualisiert
    removeSuccess: Dev-Extension erfolgreich entfernt
    errors:
      reloadFailed: Extension konnte nicht neu geladen werden
      removeFailed: Extension konnte nicht entfernt werden

en:
  title: Developer Settings
  description: Load extensions in development mode for faster testing with hot-reload.
  add:
    title: Add Dev Extension
    extensionPath: Extension Path
    extensionPathPlaceholder: /path/to/your/extension
    extensionPathHint: Path to your extension project (contains haextension/ and haextension.json)
    browse: Browse
    browseTitle: Select Extension Directory
    loadExtension: Load Extension
    success: Dev extension loaded successfully
    errors:
      browseFailed: Failed to select directory
      loadFailed: Failed to load extension
  list:
    title: Loaded Dev Extensions
    empty: No dev extensions loaded
    reload: Reload
    remove: Remove
    reloadInfo: Extension will be automatically updated on next load
    removeSuccess: Dev extension removed successfully
    errors:
      reloadFailed: Failed to reload extension
      removeFailed: Failed to remove extension
</i18n>

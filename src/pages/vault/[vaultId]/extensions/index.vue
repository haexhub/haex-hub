<template>
  <div class="flex flex-col h-full">
    <!-- Header with Actions -->
    <div
      class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-6 border-b border-gray-200 dark:border-gray-800"
    >
      <div>
        <h1 class="text-2xl font-bold">
          {{ t('title') }}
        </h1>
        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
          {{ t('subtitle') }}
        </p>
      </div>

      <div
        class="flex flex-col sm:flex-row items-stretch sm:items-center gap-3"
      >
        <!-- Marketplace Selector -->
        <USelectMenu
          v-model="selectedMarketplace"
          :items="marketplaces"
          value-key="id"
          class="w-full sm:w-48"
        >
          <template #leading>
            <UIcon name="i-heroicons-building-storefront" />
          </template>
        </USelectMenu>

        <!-- Install from File Button -->
        <UiButton
          :label="t('extension.installFromFile')"
          icon="i-heroicons-arrow-up-tray"
          color="neutral"
          @click="onSelectExtensionAsync"
        />
      </div>
    </div>

    <!-- Search and Filters -->
    <div
      class="flex flex-col sm:flex-row items-stretch sm:items-center gap-4 p-6 border-b border-gray-200 dark:border-gray-800"
    >
      <UInput
        v-model="searchQuery"
        :placeholder="t('search.placeholder')"
        icon="i-heroicons-magnifying-glass"
        class="flex-1"
      />
      <USelectMenu
        v-model="selectedCategory"
        :items="categories"
        :placeholder="t('filter.category')"
        value-key="id"
        class="w-full sm:w-48"
      >
        <template #leading>
          <UIcon name="i-heroicons-tag" />
        </template>
      </USelectMenu>
    </div>

    <!-- Extensions Grid -->
    <div class="flex-1 overflow-auto p-6">
      <div
        v-if="filteredExtensions.length"
        class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
      >
        <!-- Marketplace Extension Card -->
        <HaexExtensionMarketplaceCard
          v-for="ext in filteredExtensions"
          :key="ext.id"
          :extension="ext"
          @install="onInstallFromMarketplace(ext)"
          @details="onShowExtensionDetails(ext)"
        />
      </div>

      <!-- Empty State -->
      <div
        v-else
        class="flex flex-col items-center justify-center h-full text-center"
      >
        <UIcon
          name="i-heroicons-magnifying-glass"
          class="w-16 h-16 text-gray-400 mb-4"
        />
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
          {{ t('empty.title') }}
        </h3>
        <p class="text-gray-500 dark:text-gray-400 mt-2">
          {{ t('empty.description') }}
        </p>
      </div>
    </div>

    <HaexExtensionDialogReinstall
      v-model:open="openOverwriteDialog"
      v-model:preview="preview"
      @confirm="reinstallExtensionAsync"
    />

    <HaexExtensionDialogInstall
      v-model:open="showConfirmation"
      :preview="preview"
      @confirm="(addToDesktop) => addExtensionAsync(addToDesktop)"
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
  IMarketplaceExtension,
} from '~/types/haexhub'
import { open } from '@tauri-apps/plugin-dialog'
import type { ExtensionPreview } from '~~/src-tauri/bindings/ExtensionPreview'

definePageMeta({
  name: 'extensionOverview',
})

const { t } = useI18n()
const extensionStore = useExtensionsStore()
const desktopStore = useDesktopStore()

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

// Marketplace State
const selectedMarketplace = ref('official')
const searchQuery = ref('')
const selectedCategory = ref('all')

// Marketplaces (später von API laden)
const marketplaces = [
  {
    id: 'official',
    label: t('marketplace.official'),
    icon: 'i-heroicons-building-storefront',
  },
  {
    id: 'community',
    label: t('marketplace.community'),
    icon: 'i-heroicons-users',
  },
]

// Categories
const categories = computed(() => [
  { id: 'all', label: t('category.all') },
  { id: 'productivity', label: t('category.productivity') },
  { id: 'security', label: t('category.security') },
  { id: 'utilities', label: t('category.utilities') },
  { id: 'integration', label: t('category.integration') },
])

// Dummy Marketplace Extensions (später von API laden)
const marketplaceExtensions = ref<IMarketplaceExtension[]>([
  {
    id: 'haex-passy',
    name: 'HaexPassDummy',
    version: '1.0.0',
    author: 'HaexHub Team',
    public_key: 'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2',
    description:
      'Sicherer Passwort-Manager mit Ende-zu-Ende-Verschlüsselung und Autofill-Funktion.',
    icon: 'i-heroicons-lock-closed',
    homepage: null,
    downloads: 15420,
    rating: 4.8,
    verified: true,
    tags: ['security', 'password', 'productivity'],
    category: 'security',
    downloadUrl: '/extensions/haex-pass-1.0.0.haextension',
    isInstalled: false,
  },
  {
    id: 'haex-notes',
    name: 'HaexNotes',
    version: '2.1.0',
    author: 'HaexHub Team',
    public_key: 'b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3',
    description:
      'Markdown-basierter Notizen-Editor mit Syntax-Highlighting und Live-Preview.',
    icon: 'i-heroicons-document-text',
    homepage: null,
    downloads: 8930,
    rating: 4.5,
    verified: true,
    tags: ['productivity', 'notes', 'markdown'],
    category: 'productivity',
    downloadUrl: '/extensions/haex-notes-2.1.0.haextension',
    isInstalled: false,
  },
  {
    id: 'haex-backup',
    name: 'HaexBackup',
    version: '1.5.2',
    author: 'Community',
    public_key: 'c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4',
    description:
      'Automatische Backups deiner Daten mit Cloud-Sync-Unterstützung.',
    icon: 'i-heroicons-cloud-arrow-up',
    homepage: null,
    downloads: 5240,
    rating: 4.6,
    verified: false,
    tags: ['backup', 'cloud', 'utilities'],
    category: 'utilities',
    downloadUrl: '/extensions/haex-backup-1.5.2.haextension',
    isInstalled: false,
  },
  {
    id: 'haex-calendar',
    name: 'HaexCalendar',
    version: '3.0.1',
    author: 'HaexHub Team',
    public_key: 'd4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5',
    description:
      'Integrierter Kalender mit Event-Management und Synchronisation.',
    icon: 'i-heroicons-calendar',
    homepage: null,
    downloads: 12100,
    rating: 4.7,
    verified: true,
    tags: ['productivity', 'calendar', 'events'],
    category: 'productivity',
    downloadUrl: '/extensions/haex-calendar-3.0.1.haextension',
    isInstalled: false,
  },
  {
    id: 'haex-2fa',
    name: 'Haex2FA',
    version: '1.2.0',
    author: 'Security Team',
    public_key: 'e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6',
    description:
      '2-Faktor-Authentifizierung Manager mit TOTP und Backup-Codes.',
    icon: 'i-heroicons-shield-check',
    homepage: null,
    downloads: 7800,
    rating: 4.9,
    verified: true,
    tags: ['security', '2fa', 'authentication'],
    category: 'security',
    downloadUrl: '/extensions/haex-2fa-1.2.0.haextension',
    isInstalled: false,
  },
  {
    id: 'haex-github',
    name: 'GitHub Integration',
    version: '1.0.5',
    author: 'Community',
    public_key: 'f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7',
    description:
      'Direkter Zugriff auf GitHub Repositories, Issues und Pull Requests.',
    icon: 'i-heroicons-code-bracket',
    homepage: null,
    downloads: 4120,
    rating: 4.3,
    verified: false,
    tags: ['integration', 'github', 'development'],
    category: 'integration',
    downloadUrl: '/extensions/haex-github-1.0.5.haextension',
    isInstalled: false,
  },
])

// Mark marketplace extensions as installed if they exist in availableExtensions
const allExtensions = computed((): IMarketplaceExtension[] => {
  return marketplaceExtensions.value.map((ext) => {
    // Extensions are uniquely identified by public_key + name
    const installedExt = extensionStore.availableExtensions.find((installed) => {
      return installed.publicKey === ext.publicKey && installed.name === ext.name
    })

    if (installedExt) {
      return {
        ...ext,
        isInstalled: true,
        // Show installed version if it differs from marketplace version
        installedVersion: installedExt.version !== ext.version ? installedExt.version : undefined,
      }
    }

    return {
      ...ext,
      isInstalled: false,
      installedVersion: undefined,
    }
  })
})

// Filtered Extensions
const filteredExtensions = computed(() => {
  return allExtensions.value.filter((ext) => {
    const matchesSearch =
      !searchQuery.value ||
      ext.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      ext.description?.toLowerCase().includes(searchQuery.value.toLowerCase())

    const matchesCategory =
      selectedCategory.value === 'all' ||
      ext.category === selectedCategory.value

    return matchesSearch && matchesCategory
  })
})

// Install from marketplace
const onInstallFromMarketplace = async (ext: unknown) => {
  console.log('Install from marketplace:', ext)
  // TODO: Download extension from marketplace and install
  add({ color: 'info', description: t('extension.marketplace.comingSoon') })
}

// Show extension details
const onShowExtensionDetails = (ext: unknown) => {
  console.log('Show details:', ext)
  // TODO: Show extension details modal
}

const onSelectExtensionAsync = async () => {
  try {
    extension.path = await open({ directory: false, recursive: true })
    if (!extension.path) return

    preview.value = await extensionStore.previewManifestAsync(extension.path)

    if (!preview.value?.manifest) return

    // Check if already installed using public_key + name
    const isAlreadyInstalled = extensionStore.availableExtensions.some(
      (ext) =>
        ext.publicKey === preview.value!.manifest.public_key &&
        ext.name === preview.value!.manifest.name
    )

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

const addExtensionAsync = async (addToDesktop: boolean = false) => {
  try {
    console.log(
      'preview.value?.editable_permissions',
      preview.value?.editable_permissions,
    )
    const extensionId = await extensionStore.installAsync(
      extension.path,
      preview.value?.editable_permissions,
    )
    await extensionStore.loadExtensionsAsync()

    // Add to desktop if requested
    if (addToDesktop && extensionId) {
      await desktopStore.addDesktopItemAsync('extension', extensionId, 50, 50)
    }

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

const reinstallExtensionAsync = async () => {
  try {
    if (!preview.value?.manifest) return

    // Find the installed extension to get its current version
    const installedExt = extensionStore.availableExtensions.find(
      (ext) =>
        ext.publicKey === preview.value!.manifest.public_key &&
        ext.name === preview.value!.manifest.name
    )

    if (installedExt) {
      // Remove old extension first
      await extensionStore.removeExtensionAsync(
        installedExt.publicKey,
        installedExt.name,
        installedExt.version
      )
    }

    // Then install new version
    await addExtensionAsync()
  } catch (error) {
    console.error('Fehler reinstallExtensionAsync:', error)
    add({ color: 'error', description: JSON.stringify(error) })
    await addNotificationAsync({ text: JSON.stringify(error), type: 'error' })
  }
}

const extensionToBeRemoved = ref<IHaexHubExtension>()
const showRemoveDialog = ref(false)

// Load extensions on mount
onMounted(async () => {
  try {
    await extensionStore.loadExtensionsAsync()
    console.log('Loaded extensions:', extensionStore.availableExtensions)
  } catch (error) {
    console.error('Failed to load extensions:', error)
    add({ color: 'error', description: 'Failed to load installed extensions' })
  }
})

/* const onShowRemoveDialog = (extension: IHaexHubExtension) => {
  extensionToBeRemoved.value = extension
  showRemoveDialog.value = true
} */

const removeExtensionAsync = async () => {
  if (!extensionToBeRemoved.value?.publicKey || !extensionToBeRemoved.value?.name || !extensionToBeRemoved.value?.version) {
    add({
      color: 'error',
      description: 'Erweiterung kann nicht gelöscht werden',
    })
    return
  }

  try {
    await extensionStore.removeExtensionAsync(
      extensionToBeRemoved.value.publicKey,
      extensionToBeRemoved.value.name,
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
  title: Erweiterungen
  subtitle: Entdecke und installiere Erweiterungen für HaexHub
  extension:
    installFromFile: Von Datei installieren
    add: Erweiterung hinzufügen
    success:
      title: '{extension} hinzugefügt'
      text: Die Erweiterung wurde erfolgreich hinzugefügt
    remove:
      success:
        text: 'Erweiterung {extensionName} wurde erfolgreich entfernt'
        title: '{extensionName} entfernt'
      error:
        text: "Erweiterung {extensionName} konnte nicht entfernt werden. \n {error}"
        title: 'Fehler beim Entfernen von {extensionName}'
    marketplace:
      comingSoon: Marketplace-Installation kommt bald!
  marketplace:
    official: Offizieller Marketplace
    community: Community Marketplace
  category:
    all: Alle
    productivity: Produktivität
    security: Sicherheit
    utilities: Werkzeuge
    integration: Integration
  search:
    placeholder: Erweiterungen durchsuchen...
  filter:
    category: Kategorie auswählen
  empty:
    title: Keine Erweiterungen gefunden
    description: Versuche einen anderen Suchbegriff oder eine andere Kategorie

en:
  title: Extensions
  subtitle: Discover and install extensions for HaexHub
  extension:
    installFromFile: Install from file
    add: Add Extension
    success:
      title: '{extension} added'
      text: Extension was added successfully
    remove:
      success:
        text: 'Extension {extensionName} was removed'
        title: '{extensionName} removed'
      error:
        text: "Extension {extensionName} couldn't be removed. \n {error}"
        title: 'Exception during uninstall {extensionName}'
    marketplace:
      comingSoon: Marketplace installation coming soon!
  marketplace:
    official: Official Marketplace
    community: Community Marketplace
  category:
    all: All
    productivity: Productivity
    security: Security
    utilities: Utilities
    integration: Integration
  search:
    placeholder: Search extensions...
  filter:
    category: Select category
  empty:
    title: No extensions found
    description: Try a different search term or category
</i18n>

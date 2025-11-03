<template>
  <UDrawer
    v-model:open="open"
    direction="right"
    :title="t('launcher.title')"
    :description="t('launcher.description')"
    :overlay="false"
    :modal="false"
    :handle-only="true"
    :ui="{
      content: 'w-dvw max-w-md sm:max-w-fit',
    }"
  >
    <UButton
      icon="material-symbols:apps"
      color="neutral"
      variant="outline"
      v-bind="$attrs"
      size="lg"
    />

    <template #content>
      <div class="p-4 h-full overflow-y-auto">
        <div class="flex flex-wrap">
          <!-- All launcher items (system windows + enabled extensions, alphabetically sorted) -->
          <UContextMenu
            v-for="item in launcherItems"
            :key="item.id"
            :items="getContextMenuItems(item)"
          >
            <UiButton
              square
              size="lg"
              variant="ghost"
              :ui="{
                base: 'size-24 flex flex-wrap text-sm items-center justify-center overflow-visible cursor-grab',
                leadingIcon: 'size-10',
                label: 'w-full',
              }"
              :icon="item.icon"
              :label="item.name"
              :tooltip="item.name"
              draggable="true"
              @click="openItem(item)"
              @dragstart="handleDragStart($event, item)"
              @dragend="handleDragEnd"
            />
          </UContextMenu>

          <!-- Disabled Extensions (grayed out) -->
          <UiButton
            v-for="extension in disabledExtensions"
            :key="extension.id"
            square
            size="xl"
            variant="ghost"
            :disabled="true"
            :ui="{
              base: 'size-24 flex flex-wrap text-sm items-center justify-center overflow-visible opacity-40',
              leadingIcon: 'size-10',
              label: 'w-full',
            }"
            :icon="extension.icon || 'i-heroicons-puzzle-piece-solid'"
            :label="extension.name"
            :tooltip="`${extension.name} (${t('disabled')})`"
          />
        </div>
      </div>
    </template>
  </UDrawer>

  <!-- Uninstall Confirmation Dialog -->
  <UiDialogConfirm
    v-model:open="showUninstallDialog"
    :title="t('uninstall.confirm.title')"
    :description="
      t('uninstall.confirm.description', {
        name: extensionToUninstall?.name || '',
      })
    "
    :confirm-label="t('uninstall.confirm.button')"
    confirm-icon="i-heroicons-trash"
    @confirm="confirmUninstall"
  />
</template>

<script setup lang="ts">
defineOptions({
  inheritAttrs: false,
})

const extensionStore = useExtensionsStore()
const windowManagerStore = useWindowManagerStore()

const { t } = useI18n()

const open = ref(false)

// Uninstall dialog state
const showUninstallDialog = ref(false)
const extensionToUninstall = ref<LauncherItem | null>(null)

// Unified launcher item type
interface LauncherItem {
  id: string
  name: string
  icon: string
  type: 'system' | 'extension'
}

// Combine system windows and enabled extensions, sorted alphabetically
const launcherItems = computed(() => {
  const items: LauncherItem[] = []

  // Add system windows
  const systemWindows = windowManagerStore.getAllSystemWindows()
  systemWindows.forEach((sysWin: SystemWindowDefinition) => {
    items.push({
      id: sysWin.id,
      name: sysWin.name,
      icon: sysWin.icon,
      type: 'system',
    })
  })

  // Add enabled extensions
  const enabledExtensions = extensionStore.availableExtensions.filter(
    (ext) => ext.enabled,
  )
  enabledExtensions.forEach((ext) => {
    items.push({
      id: ext.id,
      name: ext.name,
      icon: ext.icon || 'i-heroicons-puzzle-piece-solid',
      type: 'extension',
    })
  })

  // Sort alphabetically by name
  return items.sort((a, b) => a.name.localeCompare(b.name))
})

// Disabled extensions (shown grayed out at the end)
const disabledExtensions = computed(() => {
  return extensionStore.availableExtensions.filter((ext) => !ext.enabled)
})

// Open launcher item (system window or extension)
const openItem = async (item: LauncherItem) => {
  try {
    // Open the window with correct type and sourceId
    await windowManagerStore.openWindowAsync({
      sourceId: item.id,
      type: item.type,
      icon: item.icon,
      title: item.name,
    })

    open.value = false
  } catch (error) {
    console.log(error)
  }
}

// Uninstall extension - shows confirmation dialog first
const uninstallExtension = async (item: LauncherItem) => {
  extensionToUninstall.value = item
  showUninstallDialog.value = true
}

// Confirm uninstall - actually removes the extension
const confirmUninstall = async () => {
  if (!extensionToUninstall.value) return

  try {
    const extension = extensionStore.availableExtensions.find(
      (ext) => ext.id === extensionToUninstall.value!.id,
    )
    if (!extension) return

    // Close all windows of this extension first
    const extensionWindows = windowManagerStore.windows.filter(
      (win) => win.type === 'extension' && win.sourceId === extension.id,
    )

    for (const win of extensionWindows) {
      windowManagerStore.closeWindow(win.id)
    }

    // Uninstall the extension
    await extensionStore.removeExtensionAsync(
      extension.publicKey,
      extension.name,
      extension.version,
    )

    // Refresh available extensions list
    await extensionStore.loadExtensionsAsync()

    // Close dialog and reset state
    showUninstallDialog.value = false
    extensionToUninstall.value = null
  } catch (error) {
    console.error('Failed to uninstall extension:', error)
  }
}

// Get context menu items for launcher item
const getContextMenuItems = (item: LauncherItem) => {
  const items = [
    {
      label: t('contextMenu.open'),
      icon: 'i-heroicons-arrow-top-right-on-square',
      onSelect: () => openItem(item),
    },
  ]

  // Add uninstall option for extensions
  if (item.type === 'extension') {
    items.push({
      label: t('contextMenu.uninstall'),
      icon: 'i-heroicons-trash',
      onSelect: () => uninstallExtension(item),
    })
  }

  return items
}

// Drag & Drop handling
const handleDragStart = (event: DragEvent, item: LauncherItem) => {
  if (!event.dataTransfer) return

  // Store the launcher item data
  event.dataTransfer.effectAllowed = 'copy'
  event.dataTransfer.setData(
    'application/haex-launcher-item',
    JSON.stringify(item),
  )

  // Set drag image (optional - uses default if not set)
  const dragImage = event.target as HTMLElement
  if (dragImage) {
    event.dataTransfer.setDragImage(dragImage, 20, 20)
  }
}
</script>

<i18n lang="yaml">
de:
  disabled: Deaktiviert
  marketplace: Marketplace
  launcher:
    title: App Launcher
    description: Wähle eine App zum Öffnen
  contextMenu:
    open: Öffnen
    uninstall: Deinstallieren
  uninstall:
    confirm:
      title: Erweiterung deinstallieren
      description: Möchtest du wirklich "{name}" deinstallieren? Diese Aktion kann nicht rückgängig gemacht werden.
      button: Deinstallieren

en:
  disabled: Disabled
  marketplace: Marketplace
  launcher:
    title: App Launcher
    description: Select an app to open
  contextMenu:
    open: Open
    uninstall: Uninstall
  uninstall:
    confirm:
      title: Uninstall Extension
      description: Do you really want to uninstall "{name}"? This action cannot be undone.
      button: Uninstall
</i18n>

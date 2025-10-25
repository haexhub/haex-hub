<template>
  <UPopover v-model:open="open">
    <UButton
      icon="material-symbols:apps"
      color="neutral"
      variant="outline"
      v-bind="$attrs"
      size="lg"
    />

    <template #content>
      <ul class="p-4 max-h-96 grid grid-cols-3 gap-2 overflow-scroll">
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
              base: 'size-24 flex flex-wrap text-sm items-center justify-center overflow-visible cursor-grab active:cursor-grabbing',
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
      </ul>
    </template>
  </UPopover>
</template>

<script setup lang="ts">
const extensionStore = useExtensionsStore()
const windowManagerStore = useWindowManagerStore()

const { t } = useI18n()

const open = ref(false)

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

// Uninstall extension
const uninstallExtension = async (item: LauncherItem) => {
  try {
    const extension = extensionStore.availableExtensions.find(ext => ext.id === item.id)
    if (!extension) return

    await extensionStore.removeExtensionAsync(
      extension.publicKey,
      extension.name,
      extension.version
    )
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
      click: () => openItem(item),
    }
  ]

  // Add uninstall option for extensions
  if (item.type === 'extension') {
    items.push({
      label: t('contextMenu.uninstall'),
      icon: 'i-heroicons-trash',
      click: () => uninstallExtension(item),
    })
  }

  return items
}

// Drag & Drop handling
const handleDragStart = (event: DragEvent, item: LauncherItem) => {
  if (!event.dataTransfer) return

  // Store the launcher item data
  event.dataTransfer.effectAllowed = 'copy'
  event.dataTransfer.setData('application/haex-launcher-item', JSON.stringify(item))

  // Set drag image (optional - uses default if not set)
  const dragImage = event.target as HTMLElement
  if (dragImage) {
    event.dataTransfer.setDragImage(dragImage, 20, 20)
  }
}

const handleDragEnd = () => {
  // Cleanup if needed
}
</script>

<i18n lang="yaml">
de:
  disabled: Deaktiviert
  marketplace: Marketplace
  contextMenu:
    open: Öffnen
    uninstall: Deinstallieren

en:
  disabled: Disabled
  marketplace: Marketplace
  contextMenu:
    open: Open
    uninstall: Uninstall
</i18n>

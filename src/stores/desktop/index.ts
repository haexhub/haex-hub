import { eq } from 'drizzle-orm'
import { haexDesktopItems, haexDevices } from '~/database/schemas'
import type {
  InsertHaexDesktopItems,
  SelectHaexDesktopItems,
} from '~/database/schemas'
import {
  DesktopIconSizePreset,
  iconSizePresetValues,
} from '~/stores/vault/settings'
import de from './de.json'
import en from './en.json'

export type DesktopItemType = 'extension' | 'file' | 'folder' | 'system'

export interface IDesktopItem extends SelectHaexDesktopItems {
  label?: string
  icon?: string
  referenceId: string // Computed: extensionId or systemWindowId
}

export const useDesktopStore = defineStore('desktopStore', () => {
  const { currentVault } = storeToRefs(useVaultStore())
  const workspaceStore = useWorkspaceStore()
  const { currentWorkspace } = storeToRefs(workspaceStore)
  const { $i18n } = useNuxtApp()
  const uiStore = useUiStore()
  const { isSmallScreen } = storeToRefs(uiStore)
  const deviceStore = useDeviceStore()
  const settingsStore = useVaultSettingsStore()

  $i18n.setLocaleMessage('de', {
    desktop: de,
  })
  $i18n.setLocaleMessage('en', { desktop: en })

  const desktopItems = ref<IDesktopItem[]>([])
  const selectedItemIds = ref<Set<string>>(new Set())

  // Desktop Grid Settings (stored in DB per device)
  const iconSizePreset = ref<DesktopIconSizePreset>(DesktopIconSizePreset.medium)

  // Get device internal ID from DB
  const getDeviceInternalIdAsync = async () => {
    if (!deviceStore.deviceId || !currentVault.value?.drizzle) return undefined

    const device = await currentVault.value.drizzle.query.haexDevices.findFirst({
      where: eq(haexDevices.deviceId, deviceStore.deviceId),
    })

    return device?.id ? device.id : undefined
  }

  // Sync icon size from DB
  const syncDesktopIconSizeAsync = async () => {
    const deviceInternalId = await getDeviceInternalIdAsync()
    if (!deviceInternalId) return

    const preset = await settingsStore.syncDesktopIconSizeAsync(deviceInternalId)
    iconSizePreset.value = preset
  }

  // Update icon size in DB
  const updateDesktopIconSizeAsync = async (preset: DesktopIconSizePreset) => {
    const deviceInternalId = await getDeviceInternalIdAsync()
    if (!deviceInternalId) return

    await settingsStore.updateDesktopIconSizeAsync(deviceInternalId, preset)
    iconSizePreset.value = preset
  }

  // Reactive grid settings based on screen size
  const effectiveGridColumns = computed(() => {
    return isSmallScreen.value ? 4 : 8
  })

  const effectiveGridRows = computed(() => {
    return isSmallScreen.value ? 5 : 6
  })

  const effectiveIconSize = computed(() => {
    return iconSizePresetValues[iconSizePreset.value]
  })

  // Calculate grid cell size based on icon size
  const gridCellSize = computed(() => {
    // Add padding around icon (20px extra for spacing)
    return effectiveIconSize.value + 20
  })

  // Snap position to grid (centers icon in cell)
  // iconWidth and iconHeight are optional - if provided, they're used for centering
  const snapToGrid = (x: number, y: number, iconWidth?: number, iconHeight?: number) => {
    const cellSize = gridCellSize.value

    // Calculate which grid cell the position falls into
    const col = Math.floor(x / cellSize)
    const row = Math.floor(y / cellSize)

    // Use provided dimensions or fall back to cell size
    const actualIconWidth = iconWidth || cellSize
    const actualIconHeight = iconHeight || cellSize

    // Center the icon in the cell(s) it occupies
    const cellsWide = Math.max(1, Math.ceil(actualIconWidth / cellSize))
    const cellsHigh = Math.max(1, Math.ceil(actualIconHeight / cellSize))

    const totalWidth = cellsWide * cellSize
    const totalHeight = cellsHigh * cellSize

    const paddingX = (totalWidth - actualIconWidth) / 2
    const paddingY = (totalHeight - actualIconHeight) / 2

    return {
      x: col * cellSize + paddingX,
      y: row * cellSize + paddingY,
    }
  }

  const loadDesktopItemsAsync = async () => {
    if (!currentVault.value?.drizzle) {
      console.error('Kein Vault geöffnet')
      return
    }

    if (!currentWorkspace.value) {
      console.error('Kein Workspace aktiv')
      return
    }

    try {
      const items = await currentVault.value.drizzle
        .select()
        .from(haexDesktopItems)
        .where(eq(haexDesktopItems.workspaceId, currentWorkspace.value.id))

      desktopItems.value = items.map(item => ({
        ...item,
        referenceId: item.itemType === 'extension' ? item.extensionId! : item.systemWindowId!,
      }))
    } catch (error) {
      console.error('Fehler beim Laden der Desktop-Items:', error)
      throw error
    }
  }

  const addDesktopItemAsync = async (
    itemType: DesktopItemType,
    referenceId: string,
    positionX: number = 0,
    positionY: number = 0,
    workspaceId?: string,
  ) => {
    if (!currentVault.value?.drizzle) {
      throw new Error('Kein Vault geöffnet')
    }

    const targetWorkspaceId = workspaceId || currentWorkspace.value?.id
    if (!targetWorkspaceId) {
      throw new Error('Kein Workspace aktiv')
    }

    try {
      const newItem: InsertHaexDesktopItems = {
        workspaceId: targetWorkspaceId,
        itemType: itemType,
        extensionId: itemType === 'extension' ? referenceId : null,
        systemWindowId: itemType === 'system' || itemType === 'file' || itemType === 'folder' ? referenceId : null,
        positionX: positionX,
        positionY: positionY,
      }

      const result = await currentVault.value.drizzle
        .insert(haexDesktopItems)
        .values(newItem)
        .returning()

      if (result.length > 0 && result[0]) {
        const itemWithRef = {
          ...result[0],
          referenceId: itemType === 'extension' ? result[0].extensionId! : result[0].systemWindowId!,
        }
        desktopItems.value.push(itemWithRef)
        return itemWithRef
      }
    } catch (error) {
      console.error('Fehler beim Hinzufügen des Desktop-Items:', {
        error,
        itemType,
        referenceId,
        workspaceId: targetWorkspaceId,
        position: { x: positionX, y: positionY }
      })

      // Log full error details
      if (error && typeof error === 'object') {
        console.error('Full error object:', JSON.stringify(error, null, 2))
      }

      throw error
    }
  }

  const updateDesktopItemPositionAsync = async (
    id: string,
    positionX: number,
    positionY: number,
  ) => {
    if (!currentVault.value?.drizzle) {
      throw new Error('Kein Vault geöffnet')
    }

    try {
      const result = await currentVault.value.drizzle
        .update(haexDesktopItems)
        .set({
          positionX: positionX,
          positionY: positionY,
        })
        .where(eq(haexDesktopItems.id, id))
        .returning()

      if (result.length > 0 && result[0]) {
        const index = desktopItems.value.findIndex((item) => item.id === id)
        if (index !== -1) {
          const item = result[0]
          desktopItems.value[index] = {
            ...item,
            referenceId: item.itemType === 'extension' ? item.extensionId! : item.systemWindowId!,
          }
        }
      }
    } catch (error) {
      console.error('Fehler beim Aktualisieren der Position:', error)
      throw error
    }
  }

  const removeDesktopItemAsync = async (id: string) => {
    console.log('removeDesktopItemAsync', id)
    if (!currentVault.value?.drizzle) {
      throw new Error('Kein Vault geöffnet')
    }

    try {
      // Soft delete using haexTombstone
      await currentVault.value.drizzle
        .delete(haexDesktopItems)
        .where(eq(haexDesktopItems.id, id))

      desktopItems.value = desktopItems.value.filter((item) => item.id !== id)
    } catch (error) {
      console.error('Fehler beim Entfernen des Desktop-Items:', error)
      throw error
    }
  }

  const getDesktopItemByReference = (
    itemType: DesktopItemType,
    referenceId: string,
  ) => {
    return desktopItems.value.find(
      (item) => {
        if (item.itemType !== itemType) return false
        if (itemType === 'extension') {
          return item.extensionId === referenceId
        } else {
          return item.systemWindowId === referenceId
        }
      },
    )
  }

  const openDesktopItem = (
    itemType: DesktopItemType,
    referenceId: string,
    sourcePosition?: { x: number; y: number; width: number; height: number },
  ) => {
    const windowManager = useWindowManagerStore()

    if (itemType === 'system') {
      const systemWindow = windowManager.getAllSystemWindows().find(
        (win) => win.id === referenceId,
      )

      if (systemWindow) {
        windowManager.openWindowAsync({
          sourceId: systemWindow.id,
          type: 'system',
          icon: systemWindow.icon,
          title: systemWindow.name,
          sourcePosition,
        })
      }
    } else if (itemType === 'extension') {
      const extensionsStore = useExtensionsStore()

      const extension = extensionsStore.availableExtensions.find(
        (ext) => ext.id === referenceId,
      )

      if (extension) {
        windowManager.openWindowAsync({
          sourceId: extension.id,
          type: 'extension',
          icon: extension.icon,
          title: extension.name,
          sourcePosition,
        })
      }
    }
    // Für später: file und folder handling
  }

  const uninstallDesktopItem = async (
    id: string,
    itemType: DesktopItemType,
    referenceId: string,
  ) => {
    if (itemType === 'extension') {
      try {
        const extensionsStore = useExtensionsStore()
        const extension = extensionsStore.availableExtensions.find(
          (ext) => ext.id === referenceId,
        )
        if (!extension) {
          console.error('Extension nicht gefunden')
          return
        }

        // Uninstall the extension
        await extensionsStore.removeExtensionAsync(
          extension.publicKey,
          extension.name,
          extension.version,
        )

        // Reload extensions after uninstall
        await extensionsStore.loadExtensionsAsync()

        // Remove desktop item
        await removeDesktopItemAsync(id)
      } catch (error) {
        console.error('Fehler beim Deinstallieren:', error)
      }
    }
    // Für später: file und folder handling
  }

  const toggleSelection = (id: string, ctrlKey: boolean = false) => {
    if (ctrlKey) {
      // Mit Ctrl: Toggle einzelnes Element
      if (selectedItemIds.value.has(id)) {
        selectedItemIds.value.delete(id)
      } else {
        selectedItemIds.value.add(id)
      }
    } else {
      // Ohne Ctrl: Nur dieses Element auswählen
      selectedItemIds.value.clear()
      selectedItemIds.value.add(id)
    }
  }

  const clearSelection = () => {
    selectedItemIds.value.clear()
  }

  const isItemSelected = (id: string) => {
    return selectedItemIds.value.has(id)
  }

  const selectedItems = computed(() => {
    return desktopItems.value.filter((item) =>
      selectedItemIds.value.has(item.id),
    )
  })

  const getContextMenuItems = (
    id: string,
    itemType: DesktopItemType,
    referenceId: string,
    onUninstall: () => void,
  ) => {
    const handleOpen = () => {
      openDesktopItem(itemType, referenceId)
    }

    // Build second menu group based on item type
    const secondGroup = [
      {
        label: $i18n.t('desktop.contextMenu.removeFromDesktop'),
        icon: 'i-heroicons-x-mark',
        onSelect: async () => {
          await removeDesktopItemAsync(id)
        },
      },
    ]

    // Only show uninstall option for extensions
    if (itemType === 'extension') {
      secondGroup.push({
        label: $i18n.t('desktop.contextMenu.uninstall'),
        icon: 'i-heroicons-trash',
        onSelect: async () => {
          onUninstall()
        },
      })
    }

    return [
      [
        {
          label: $i18n.t('desktop.contextMenu.open'),
          icon: 'i-heroicons-arrow-top-right-on-square',
          onSelect: handleOpen,
        },
      ],
      secondGroup,
    ]
  }

  return {
    desktopItems,
    selectedItemIds,
    selectedItems,
    loadDesktopItemsAsync,
    addDesktopItemAsync,
    updateDesktopItemPositionAsync,
    removeDesktopItemAsync,
    getDesktopItemByReference,
    getContextMenuItems,
    openDesktopItem,
    uninstallDesktopItem,
    toggleSelection,
    clearSelection,
    isItemSelected,
    // Grid settings
    iconSizePreset,
    syncDesktopIconSizeAsync,
    updateDesktopIconSizeAsync,
    effectiveGridColumns,
    effectiveGridRows,
    effectiveIconSize,
    gridCellSize,
    snapToGrid,
  }
})

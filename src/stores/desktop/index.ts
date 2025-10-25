import { eq } from 'drizzle-orm'
import { haexDesktopItems } from '~~/src-tauri/database/schemas'
import type {
  InsertHaexDesktopItems,
  SelectHaexDesktopItems,
} from '~~/src-tauri/database/schemas'
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

  $i18n.setLocaleMessage('de', {
    desktop: de,
  })
  $i18n.setLocaleMessage('en', { desktop: en })

  const desktopItems = ref<IDesktopItem[]>([])
  const selectedItemIds = ref<Set<string>>(new Set())

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

    return [
      [
        {
          label: $i18n.t('desktop.contextMenu.open'),
          icon: 'i-heroicons-arrow-top-right-on-square',
          onSelect: handleOpen,
        },
      ],
      [
        {
          label: $i18n.t('desktop.contextMenu.removeFromDesktop'),
          icon: 'i-heroicons-x-mark',
          onSelect: async () => {
            await removeDesktopItemAsync(id)
          },
        },
        {
          label: $i18n.t('desktop.contextMenu.uninstall'),
          icon: 'i-heroicons-trash',
          onSelect: onUninstall,
        },
      ],
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
  }
})

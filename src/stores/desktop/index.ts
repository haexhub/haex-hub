import { eq } from 'drizzle-orm'
import { haexDesktopItems } from '~~/src-tauri/database/schemas'
import type {
  InsertHaexDesktopItems,
  SelectHaexDesktopItems,
} from '~~/src-tauri/database/schemas'
import de from './de.json'
import en from './en.json'

export type DesktopItemType = 'extension' | 'file' | 'folder'

export interface IDesktopItem extends SelectHaexDesktopItems {
  label?: string
  icon?: string
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

      desktopItems.value = items
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
  ) => {
    if (!currentVault.value?.drizzle) {
      throw new Error('Kein Vault geöffnet')
    }

    if (!currentWorkspace.value) {
      throw new Error('Kein Workspace aktiv')
    }

    try {
      const newItem: InsertHaexDesktopItems = {
        workspaceId: currentWorkspace.value.id,
        itemType: itemType,
        referenceId: referenceId,
        positionX: positionX,
        positionY: positionY,
      }

      const result = await currentVault.value.drizzle
        .insert(haexDesktopItems)
        .values(newItem)
        .returning()

      if (result.length > 0 && result[0]) {
        desktopItems.value.push(result[0])
        return result[0]
      }
    } catch (error) {
      console.error('Fehler beim Hinzufügen des Desktop-Items:', error)
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
          desktopItems.value[index] = result[0]
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
      (item) => item.itemType === itemType && item.referenceId === referenceId,
    )
  }

  const openDesktopItem = (
    itemType: DesktopItemType,
    referenceId: string,
    sourcePosition?: { x: number; y: number; width: number; height: number },
  ) => {
    if (itemType === 'extension') {
      const windowManager = useWindowManagerStore()
      const extensionsStore = useExtensionsStore()

      const extension = extensionsStore.availableExtensions.find(
        (ext) => ext.id === referenceId,
      )

      if (extension) {
        windowManager.openWindow(
          'extension',
          extension.id,
          extension.name,
          extension.icon || undefined,
          undefined, // Use default viewport-aware width
          undefined, // Use default viewport-aware height
          sourcePosition,
        )
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

import { eq } from 'drizzle-orm'
import { haexDesktopItems } from '~~/src-tauri/database/schemas'
import type {
  InsertHaexDesktopItems,
  SelectHaexDesktopItems,
} from '~~/src-tauri/database/schemas'

export type DesktopItemType = 'extension' | 'file' | 'folder'

export interface IDesktopItem extends SelectHaexDesktopItems {
  label?: string
  icon?: string
}

export const useDesktopStore = defineStore('desktopStore', () => {
  const { currentVault } = storeToRefs(useVaultStore())

  const desktopItems = ref<IDesktopItem[]>([])

  const loadDesktopItemsAsync = async () => {
    if (!currentVault.value?.drizzle) {
      console.error('Kein Vault geöffnet')
      return
    }

    try {
      const items = await currentVault.value.drizzle
        .select()
        .from(haexDesktopItems)

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

    try {
      const newItem: InsertHaexDesktopItems = {
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

  const getContextMenuItems = (
    id: string,
    itemType: DesktopItemType,
    referenceId: string,
    onOpen: () => void,
    onUninstall: () => void,
  ) => {
    return [
      [
        {
          label: 'Öffnen',
          icon: 'i-heroicons-arrow-top-right-on-square',
          click: onOpen,
        },
      ],
      [
        {
          label: 'Von Desktop entfernen',
          icon: 'i-heroicons-x-mark',
          click: async () => {
            await removeDesktopItemAsync(id)
          },
        },
        {
          label: 'Deinstallieren',
          icon: 'i-heroicons-trash',
          click: onUninstall,
        },
      ],
    ]
  }

  return {
    desktopItems,
    loadDesktopItemsAsync,
    addDesktopItemAsync,
    updateDesktopItemPositionAsync,
    removeDesktopItemAsync,
    getDesktopItemByReference,
    getContextMenuItems,
  }
})

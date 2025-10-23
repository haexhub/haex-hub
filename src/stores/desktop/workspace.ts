import { eq } from 'drizzle-orm'
import {
  haexWorkspaces,
  type SelectHaexWorkspaces,
} from '~~/src-tauri/database/schemas'
import type { Swiper } from 'swiper/types'

export type IWorkspace = SelectHaexWorkspaces

export const useWorkspaceStore = defineStore('workspaceStore', () => {
  const vaultStore = useVaultStore()
  const windowStore = useWindowManagerStore()

  const { currentVault } = storeToRefs(vaultStore)

  const swiperInstance = ref<Swiper | null>(null)

  const allowSwipe = ref(true)

  // Workspace Overview Mode (GNOME-style)
  const isOverviewMode = ref(false)

  const workspaces = ref<IWorkspace[]>([])

  const currentWorkspaceIndex = ref(0)

  // Load workspaces from database
  const loadWorkspacesAsync = async () => {
    if (!currentVault.value?.drizzle) {
      console.error('Kein Vault geöffnet')
      return
    }

    try {
      /* const items = await currentVault.value.drizzle
        .select()
        .from(haexWorkspaces)
        .orderBy(asc(haexWorkspaces.position))

      console.log('loadWorkspacesAsync', items)
      workspaces.value = items */

      // Create default workspace if none exist
      /* if (items.length === 0) { */
      await addWorkspaceAsync('Workspace 1')
      /* } */
    } catch (error) {
      console.error('Fehler beim Laden der Workspaces:', error)
      throw error
    }
  }

  const currentWorkspace = computed(() => {
    return workspaces.value[currentWorkspaceIndex.value]
  })

  const addWorkspaceAsync = async (name?: string) => {
    if (!currentVault.value?.drizzle) {
      throw new Error('Kein Vault geöffnet')
    }

    try {
      const newIndex = workspaces.value.length + 1
      const newWorkspace: SelectHaexWorkspaces = {
        id: crypto.randomUUID(),
        name: name || `Workspace ${newIndex}`,
        position: workspaces.value.length,
        haexTimestamp: '',
        haexTombstone: false,
      }
      workspaces.value.push(newWorkspace)
      currentWorkspaceIndex.value = workspaces.value.length - 1
      /* const result = await currentVault.value.drizzle
        .insert(haexWorkspaces)
        .values(newWorkspace)
        .returning()

      if (result.length > 0 && result[0]) {
        workspaces.value.push(result[0])
        currentWorkspaceIndex.value = workspaces.value.length - 1
        return result[0]
      } */
    } catch (error) {
      console.error('Fehler beim Hinzufügen des Workspace:', error)
      throw error
    }
  }

  const closeWorkspaceAsync = async (workspaceId: string) => {
    const openWindows = windowStore.windowsByWorkspaceId(workspaceId)

    for (const window of openWindows.value) {
      windowStore.closeWindow(window.id)
    }

    return await removeWorkspaceAsync(workspaceId)
  }

  const removeWorkspaceAsync = async (workspaceId: string) => {
    // Don't allow removing the last workspace
    if (workspaces.value.length <= 1) return

    if (!currentVault.value?.drizzle) {
      throw new Error('Kein Vault geöffnet')
    }

    const index = workspaces.value.findIndex((ws) => ws.id === workspaceId)
    if (index === -1) return

    workspaces.value.splice(index, 1)
    workspaces.value.forEach((workspace, index) => (workspace.position = index))

    try {
      /* await currentVault.value.drizzle.transaction(async (tx) => {
        await tx
          .delete(haexWorkspaces)
          .where(eq(haexWorkspaces.id, workspaceId))

        workspaces.value.splice(index, 1)
        workspaces.value.forEach(
          (workspace, index) => (workspace.position = index),
        )

        for (const workspace of workspaces.value) {
          await tx
            .update(haexWorkspaces)
            .set({ position: index })
            .where(eq(haexWorkspaces.position, workspace.position))
        }
      }) */

      // Adjust current index if needed
      if (currentWorkspaceIndex.value >= workspaces.value.length) {
        currentWorkspaceIndex.value = workspaces.value.length - 1
      }
    } catch (error) {
      console.error('Fehler beim Entfernen des Workspace:', error)
      throw error
    }
  }

  const switchToWorkspace = (workspaceId?: string) => {
    const workspace = workspaces.value.find((w) => w.id === workspaceId)

    console.log('switchToWorkspace', workspace)
    if (workspace) {
      currentWorkspaceIndex.value = workspace?.position
    } else {
      currentWorkspaceIndex.value = 0
    }

    return currentWorkspaceIndex.value
  }

  const switchToNext = () => {
    if (currentWorkspaceIndex.value < workspaces.value.length - 1) {
      currentWorkspaceIndex.value++
    }
  }

  const switchToPrevious = () => {
    if (currentWorkspaceIndex.value > 0) {
      currentWorkspaceIndex.value--
    }
  }

  const renameWorkspaceAsync = async (workspaceId: string, newName: string) => {
    if (!currentVault.value?.drizzle) {
      throw new Error('Kein Vault geöffnet')
    }

    try {
      const result = await currentVault.value.drizzle
        .update(haexWorkspaces)
        .set({ name: newName })
        .where(eq(haexWorkspaces.id, workspaceId))
        .returning()

      if (result.length > 0 && result[0]) {
        const index = workspaces.value.findIndex((ws) => ws.id === workspaceId)
        if (index !== -1) {
          workspaces.value[index] = result[0]
        }
      }
    } catch (error) {
      console.error('Fehler beim Umbenennen des Workspace:', error)
      throw error
    }
  }

  const slideToWorkspace = (workspaceId?: string) => {
    const index = switchToWorkspace(workspaceId)
    if (swiperInstance.value) {
      swiperInstance.value.slideTo(index)
    }
    isOverviewMode.value = false
  }

  return {
    addWorkspaceAsync,
    allowSwipe,
    closeWorkspaceAsync,
    currentWorkspace,
    currentWorkspaceIndex,
    isOverviewMode,
    slideToWorkspace,
    loadWorkspacesAsync,
    removeWorkspaceAsync,
    renameWorkspaceAsync,
    swiperInstance,
    switchToNext,
    switchToPrevious,
    switchToWorkspace,
    workspaces,
  }
})

import { eq } from 'drizzle-orm'
import {
  haexSyncBackends,
  type InsertHaexSyncBackends,
  type SelectHaexSyncBackends,
} from '~/database/schemas'

export const useSyncBackendsStore = defineStore('syncBackendsStore', () => {
  const { currentVault } = storeToRefs(useVaultStore())

  const backends = ref<SelectHaexSyncBackends[]>([])

  const enabledBackends = computed(() =>
    backends.value.filter((b) => b.enabled),
  )

  const sortedBackends = computed(() =>
    [...backends.value].sort((a, b) => (b.priority || 0) - (a.priority || 0)),
  )

  // Load all sync backends from database
  const loadBackendsAsync = async () => {
    if (!currentVault.value?.drizzle) {
      console.error('No vault opened')
      return
    }

    try {
      const result = await currentVault.value.drizzle
        .select()
        .from(haexSyncBackends)

      backends.value = result
    } catch (error) {
      console.error('Failed to load sync backends:', error)
      throw error
    }
  }

  // Add a new sync backend
  const addBackendAsync = async (backend: InsertHaexSyncBackends) => {
    if (!currentVault.value?.drizzle) {
      throw new Error('No vault opened')
    }

    try {
      const result = await currentVault.value.drizzle
        .insert(haexSyncBackends)
        .values(backend)
        .returning()

      if (result.length > 0 && result[0]) {
        backends.value.push(result[0])
        return result[0]
      }
    } catch (error) {
      console.error('Failed to add sync backend:', error)
      throw error
    }
  }

  // Update an existing sync backend
  const updateBackendAsync = async (
    id: string,
    updates: Partial<InsertHaexSyncBackends>,
  ) => {
    if (!currentVault.value?.drizzle) {
      throw new Error('No vault opened')
    }

    try {
      const result = await currentVault.value.drizzle
        .update(haexSyncBackends)
        .set(updates)
        .where(eq(haexSyncBackends.id, id))
        .returning()

      if (result.length > 0 && result[0]) {
        const index = backends.value.findIndex((b) => b.id === id)
        if (index !== -1) {
          backends.value[index] = result[0]
        }
        return result[0]
      }
    } catch (error) {
      console.error('Failed to update sync backend:', error)
      throw error
    }
  }

  // Delete a sync backend
  const deleteBackendAsync = async (id: string) => {
    if (!currentVault.value?.drizzle) {
      throw new Error('No vault opened')
    }

    try {
      await currentVault.value.drizzle
        .delete(haexSyncBackends)
        .where(eq(haexSyncBackends.id, id))

      backends.value = backends.value.filter((b) => b.id !== id)
    } catch (error) {
      console.error('Failed to delete sync backend:', error)
      throw error
    }
  }

  // Enable/disable a backend
  const toggleBackendAsync = async (id: string, enabled: boolean) => {
    return updateBackendAsync(id, { enabled })
  }

  // Update backend priority (for sync order)
  const updatePriorityAsync = async (id: string, priority: number) => {
    return updateBackendAsync(id, { priority })
  }

  return {
    backends,
    enabledBackends,
    sortedBackends,
    loadBackendsAsync,
    addBackendAsync,
    updateBackendAsync,
    deleteBackendAsync,
    toggleBackendAsync,
    updatePriorityAsync,
  }
})

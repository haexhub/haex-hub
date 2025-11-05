/**
 * Sync Orchestrator Store - Orchestrates sync operations across all backends
 * Uses Supabase Realtime subscriptions for instant sync
 */

import { eq, gt } from 'drizzle-orm'
import type { RealtimeChannel } from '@supabase/supabase-js'
import {
  haexCrdtLogs,
  haexSyncStatus,
  type SelectHaexCrdtLogs,
  type SelectHaexSyncStatus,
} from '~/database/schemas'

interface SyncState {
  isConnected: boolean
  isSyncing: boolean
  error: string | null
  subscription: RealtimeChannel | null
  status: SelectHaexSyncStatus | null
}

interface BackendSyncState {
  [backendId: string]: SyncState
}

export const useSyncOrchestratorStore = defineStore(
  'syncOrchestratorStore',
  () => {
    const { currentVault, currentVaultId } = storeToRefs(useVaultStore())
    const syncBackendsStore = useSyncBackendsStore()
    const syncEngineStore = useSyncEngineStore()

    // Sync state per backend
    const syncStates = ref<BackendSyncState>({})

    // Track if we're currently processing a local write
    const isProcessingLocalWrite = ref(false)

    /**
     * Loads sync status from database for a backend
     */
    const loadSyncStatusAsync = async (
      backendId: string,
    ): Promise<SelectHaexSyncStatus | null> => {
      if (!currentVault.value?.drizzle) {
        throw new Error('No vault opened')
      }

      try {
        const results = await currentVault.value.drizzle
          .select()
          .from(haexSyncStatus)
          .where(eq(haexSyncStatus.backendId, backendId))
          .limit(1)

        return results[0] ?? null
      } catch (error) {
        console.error('Failed to load sync status:', error)
        return null
      }
    }

    /**
     * Updates sync status in database
     */
    const updateSyncStatusAsync = async (
      backendId: string,
      updates: Partial<SelectHaexSyncStatus>,
    ): Promise<void> => {
      if (!currentVault.value?.drizzle) {
        throw new Error('No vault opened')
      }

      try {
        const existing = await loadSyncStatusAsync(backendId)

        if (existing) {
          // Update existing
          await currentVault.value.drizzle
            .update(haexSyncStatus)
            .set({
              ...updates,
              lastSyncAt: new Date().toISOString(),
            })
            .where(eq(haexSyncStatus.backendId, backendId))
        } else {
          // Insert new
          await currentVault.value.drizzle.insert(haexSyncStatus).values({
            backendId,
            ...updates,
            lastSyncAt: new Date().toISOString(),
          })
        }

        // Update local state
        if (syncStates.value[backendId]) {
          syncStates.value[backendId].status = await loadSyncStatusAsync(
            backendId,
          )
        }
      } catch (error) {
        console.error('Failed to update sync status:', error)
        throw error
      }
    }

    /**
     * Gets logs that need to be pushed to server (after last push HLC)
     */
    const getLogsToPushAsync = async (
      backendId: string,
    ): Promise<SelectHaexCrdtLogs[]> => {
      if (!currentVault.value?.drizzle) {
        throw new Error('No vault opened')
      }

      try {
        const status = await loadSyncStatusAsync(backendId)
        const lastPushHlc = status?.lastPushHlcTimestamp

        const query = currentVault.value.drizzle
          .select()
          .from(haexCrdtLogs)
          .orderBy(haexCrdtLogs.haexTimestamp)

        if (lastPushHlc) {
          return await query.where(
            gt(haexCrdtLogs.haexTimestamp, lastPushHlc),
          )
        }

        return await query
      } catch (error) {
        console.error('Failed to get logs to push:', error)
        throw error
      }
    }

    /**
     * Applies remote logs to local database
     */
    const applyRemoteLogsAsync = async (
      logs: SelectHaexCrdtLogs[],
    ): Promise<void> => {
      if (!currentVault.value?.drizzle) {
        throw new Error('No vault opened')
      }

      try {
        // Insert logs into local CRDT log table
        for (const log of logs) {
          await currentVault.value.drizzle
            .insert(haexCrdtLogs)
            .values(log)
            .onConflictDoNothing() // Skip if already exists
        }

        // TODO: Apply CRDT log entries to actual data tables
        // This requires replaying the operations from the log
        console.log(`Applied ${logs.length} remote logs to local database`)
      } catch (error) {
        console.error('Failed to apply remote logs:', error)
        throw error
      }
    }

    /**
     * Pushes local changes to a specific backend
     */
    const pushToBackendAsync = async (backendId: string): Promise<void> => {
      if (!currentVaultId.value) {
        throw new Error('No vault opened')
      }

      const state = syncStates.value[backendId]
      if (!state) {
        throw new Error('Backend not initialized')
      }

      if (state.isSyncing) {
        console.log(`Already syncing with backend ${backendId}`)
        return
      }

      state.isSyncing = true
      state.error = null

      try {
        // Get logs that need to be pushed
        const logs = await getLogsToPushAsync(backendId)

        if (logs.length === 0) {
          console.log(`No logs to push to backend ${backendId}`)
          return
        }

        await syncEngineStore.pushLogsAsync(
          backendId,
          currentVaultId.value,
          logs,
        )

        // Update sync status with last pushed HLC timestamp
        const lastHlc = logs[logs.length - 1]?.haexTimestamp
        if (lastHlc) {
          await updateSyncStatusAsync(backendId, {
            lastPushHlcTimestamp: lastHlc,
          })
        }

        console.log(`Pushed ${logs.length} logs to backend ${backendId}`)
      } catch (error) {
        console.error(`Failed to push to backend ${backendId}:`, error)
        state.error = error instanceof Error ? error.message : 'Unknown error'
        await updateSyncStatusAsync(backendId, {
          error: state.error,
        })
        throw error
      } finally {
        state.isSyncing = false
      }
    }

    /**
     * Pulls changes from a specific backend
     */
    const pullFromBackendAsync = async (backendId: string): Promise<void> => {
      if (!currentVaultId.value) {
        throw new Error('No vault opened')
      }

      const state = syncStates.value[backendId]
      if (!state) {
        throw new Error('Backend not initialized')
      }

      if (state.isSyncing) {
        console.log(`Already syncing with backend ${backendId}`)
        return
      }

      state.isSyncing = true
      state.error = null

      try {
        const status = await loadSyncStatusAsync(backendId)
        const afterSequence = status?.lastPullSequence ?? undefined

        const remoteLogs = await syncEngineStore.pullLogsAsync(
          backendId,
          currentVaultId.value,
          afterSequence,
          100,
        )

        if (remoteLogs.length > 0) {
          await applyRemoteLogsAsync(remoteLogs)

          // Update sync status with last pulled sequence
          // TODO: Get actual sequence from server response
          const lastSequence = Date.now()
          await updateSyncStatusAsync(backendId, {
            lastPullSequence: lastSequence,
          })

          console.log(
            `Pulled ${remoteLogs.length} logs from backend ${backendId}`,
          )
        }
      } catch (error) {
        console.error(`Failed to pull from backend ${backendId}:`, error)
        state.error = error instanceof Error ? error.message : 'Unknown error'
        await updateSyncStatusAsync(backendId, {
          error: state.error,
        })
        throw error
      } finally {
        state.isSyncing = false
      }
    }

    /**
     * Handles incoming realtime changes from Supabase
     */
    const handleRealtimeChangeAsync = async (
      backendId: string,
      payload: any,
    ) => {
      console.log(`Realtime change from backend ${backendId}:`, payload)

      // Don't process if we're currently writing locally to avoid loops
      if (isProcessingLocalWrite.value) {
        console.log('Skipping realtime change - local write in progress')
        return
      }

      // Pull latest changes from this backend
      try {
        await pullFromBackendAsync(backendId)
      } catch (error) {
        console.error('Failed to handle realtime change:', error)
      }
    }

    /**
     * Subscribes to realtime changes from a backend
     */
    const subscribeToBackendAsync = async (backendId: string): Promise<void> => {
      if (!currentVaultId.value) {
        throw new Error('No vault opened')
      }

      const state = syncStates.value[backendId]
      if (!state) {
        throw new Error('Backend not initialized')
      }

      if (state.subscription) {
        console.log(`Already subscribed to backend ${backendId}`)
        return
      }

      const client = syncEngineStore.supabaseClient
      if (!client) {
        throw new Error('Supabase client not initialized')
      }

      try {
        // Subscribe to sync_logs table for this vault
        const channel = client
          .channel(`sync_logs:${currentVaultId.value}`)
          .on(
            'postgres_changes',
            {
              event: 'INSERT',
              schema: 'public',
              table: 'sync_logs',
              filter: `vault_id=eq.${currentVaultId.value}`,
            },
            (payload) => {
              handleRealtimeChangeAsync(backendId, payload).catch(console.error)
            },
          )
          .subscribe((status) => {
            if (status === 'SUBSCRIBED') {
              state.isConnected = true
              console.log(`Subscribed to backend ${backendId}`)
            } else if (status === 'CHANNEL_ERROR' || status === 'TIMED_OUT') {
              state.isConnected = false
              state.error = `Subscription error: ${status}`
              console.error(
                `Subscription to backend ${backendId} failed: ${status}`,
              )
            }
          })

        state.subscription = channel
      } catch (error) {
        console.error(`Failed to subscribe to backend ${backendId}:`, error)
        state.error = error instanceof Error ? error.message : 'Unknown error'
        throw error
      }
    }

    /**
     * Unsubscribes from realtime changes
     */
    const unsubscribeFromBackendAsync = async (
      backendId: string,
    ): Promise<void> => {
      const state = syncStates.value[backendId]
      if (!state || !state.subscription) {
        return
      }

      try {
        await state.subscription.unsubscribe()
        state.subscription = null
        state.isConnected = false
        console.log(`Unsubscribed from backend ${backendId}`)
      } catch (error) {
        console.error(`Failed to unsubscribe from backend ${backendId}:`, error)
      }
    }

    /**
     * Initializes sync for a backend
     */
    const initBackendAsync = async (backendId: string): Promise<void> => {
      if (syncStates.value[backendId]) {
        console.log(`Backend ${backendId} already initialized`)
        return
      }

      // Load sync status from database
      const status = await loadSyncStatusAsync(backendId)

      // Initialize state
      syncStates.value[backendId] = {
        isConnected: false,
        isSyncing: false,
        error: null,
        subscription: null,
        status,
      }

      try {
        // Initial pull to get all existing data
        await pullFromBackendAsync(backendId)

        // Subscribe to realtime changes
        await subscribeToBackendAsync(backendId)
      } catch (error) {
        console.error(`Failed to initialize backend ${backendId}:`, error)
        throw error
      }
    }

    /**
     * Called after local write operations to push changes
     */
    const onLocalWriteAsync = async (): Promise<void> => {
      isProcessingLocalWrite.value = true

      try {
        // Push to all enabled backends in parallel
        const enabledBackends = syncBackendsStore.enabledBackends

        await Promise.allSettled(
          enabledBackends.map((backend) => pushToBackendAsync(backend.id)),
        )
      } catch (error) {
        console.error('Failed to push local changes:', error)
      } finally {
        isProcessingLocalWrite.value = false
      }
    }

    /**
     * Starts sync for all enabled backends
     */
    const startSyncAsync = async (): Promise<void> => {
      const enabledBackends = syncBackendsStore.enabledBackends

      if (enabledBackends.length === 0) {
        console.log('No enabled backends to sync with')
        return
      }

      console.log(`Starting sync with ${enabledBackends.length} backends`)

      for (const backend of enabledBackends) {
        try {
          await initBackendAsync(backend.id)
        } catch (error) {
          console.error(
            `Failed to start sync with backend ${backend.id}:`,
            error,
          )
        }
      }
    }

    /**
     * Stops sync for all backends
     */
    const stopSyncAsync = async (): Promise<void> => {
      console.log('Stopping sync for all backends')

      for (const backendId of Object.keys(syncStates.value)) {
        await unsubscribeFromBackendAsync(backendId)
      }

      syncStates.value = {}
    }

    /**
     * Gets sync state for a specific backend
     */
    const getSyncState = (backendId: string): SyncState | null => {
      return syncStates.value[backendId] ?? null
    }

    /**
     * Checks if any backend is currently syncing
     */
    const isAnySyncing = computed(() => {
      return Object.values(syncStates.value).some((state) => state.isSyncing)
    })

    /**
     * Checks if all backends are connected
     */
    const areAllConnected = computed(() => {
      const enabledBackends = syncBackendsStore.enabledBackends
      if (enabledBackends.length === 0) return false

      return enabledBackends.every((backend) => {
        const state = syncStates.value[backend.id]
        return state?.isConnected ?? false
      })
    })

    return {
      syncStates,
      isProcessingLocalWrite,
      isAnySyncing,
      areAllConnected,
      loadSyncStatusAsync,
      updateSyncStatusAsync,
      getLogsToPushAsync,
      applyRemoteLogsAsync,
      pushToBackendAsync,
      pullFromBackendAsync,
      subscribeToBackendAsync,
      unsubscribeFromBackendAsync,
      initBackendAsync,
      onLocalWriteAsync,
      startSyncAsync,
      stopSyncAsync,
      getSyncState,
    }
  },
)

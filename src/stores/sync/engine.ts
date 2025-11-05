/**
 * Sync Engine Store - Executes sync operations with haex-sync-server backends
 * Handles vault key storage and CRDT log synchronization
 */

import { createClient } from '@supabase/supabase-js'
import type { SelectHaexCrdtLogs } from '~/database/schemas'
import {
  encryptVaultKeyAsync,
  decryptVaultKeyAsync,
  encryptCrdtDataAsync,
  decryptCrdtDataAsync,
  generateVaultKey,
} from '~/utils/crypto/vaultKey'

interface VaultKeyCache {
  [vaultId: string]: {
    vaultKey: Uint8Array
    timestamp: number
  }
}

interface SyncLogData {
  vaultId: string
  encryptedData: string
  nonce: string
  haexTimestamp: string
  sequence: number
}

interface PullLogsResponse {
  logs: Array<{
    id: string
    userId: string
    vaultId: string
    encryptedData: string
    nonce: string
    haexTimestamp: string
    sequence: number
    createdAt: string
  }>
  hasMore: boolean
}

export const useSyncEngineStore = defineStore('syncEngineStore', () => {
  const { currentVault, currentVaultId } = storeToRefs(useVaultStore())
  const syncBackendsStore = useSyncBackendsStore()

  // In-memory cache for decrypted vault keys (cleared on logout/vault close)
  const vaultKeyCache = ref<VaultKeyCache>({})

  // Supabase client (initialized with config from backend)
  const supabaseClient = ref<ReturnType<typeof createClient> | null>(null)

  /**
   * Initializes Supabase client for a specific backend
   */
  const initSupabaseClientAsync = async (backendId: string) => {
    const backend = syncBackendsStore.backends.find((b) => b.id === backendId)
    if (!backend) {
      throw new Error('Backend not found')
    }

    // Get Supabase URL and anon key from server health check
    const response = await fetch(backend.serverUrl)
    if (!response.ok) {
      throw new Error('Failed to connect to sync server')
    }

    const serverInfo = await response.json()
    const supabaseUrl = serverInfo.supabaseUrl

    // For now, we need to configure the anon key somewhere
    // TODO: Store this in backend config or fetch from somewhere secure
    const supabaseAnonKey = 'YOUR_SUPABASE_ANON_KEY'

    supabaseClient.value = createClient(supabaseUrl, supabaseAnonKey)
  }

  /**
   * Gets the current Supabase auth token
   */
  const getAuthTokenAsync = async (): Promise<string | null> => {
    if (!supabaseClient.value) {
      return null
    }

    const {
      data: { session },
    } = await supabaseClient.value.auth.getSession()
    return session?.access_token ?? null
  }

  /**
   * Stores encrypted vault key on the server
   */
  const storeVaultKeyAsync = async (
    backendId: string,
    vaultId: string,
    password: string,
  ): Promise<void> => {
    const backend = syncBackendsStore.backends.find((b) => b.id === backendId)
    if (!backend) {
      throw new Error('Backend not found')
    }

    // Generate new vault key
    const vaultKey = generateVaultKey()

    // Encrypt vault key with password
    const encryptedData = await encryptVaultKeyAsync(vaultKey, password)

    // Get auth token
    const token = await getAuthTokenAsync()
    if (!token) {
      throw new Error('Not authenticated')
    }

    // Send to server
    const response = await fetch(`${backend.serverUrl}/sync/vault-key`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({
        vaultId,
        ...encryptedData,
      }),
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({}))
      throw new Error(
        `Failed to store vault key: ${error.error || response.statusText}`,
      )
    }

    // Cache decrypted vault key
    vaultKeyCache.value[vaultId] = {
      vaultKey,
      timestamp: Date.now(),
    }
  }

  /**
   * Retrieves and decrypts vault key from the server
   */
  const getVaultKeyAsync = async (
    backendId: string,
    vaultId: string,
    password: string,
  ): Promise<Uint8Array> => {
    // Check cache first
    const cached = vaultKeyCache.value[vaultId]
    if (cached) {
      return cached.vaultKey
    }

    const backend = syncBackendsStore.backends.find((b) => b.id === backendId)
    if (!backend) {
      throw new Error('Backend not found')
    }

    // Get auth token
    const token = await getAuthTokenAsync()
    if (!token) {
      throw new Error('Not authenticated')
    }

    // Fetch from server
    const response = await fetch(
      `${backend.serverUrl}/sync/vault-key/${vaultId}`,
      {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      },
    )

    if (response.status === 404) {
      throw new Error('Vault key not found on server')
    }

    if (!response.ok) {
      const error = await response.json().catch(() => ({}))
      throw new Error(
        `Failed to get vault key: ${error.error || response.statusText}`,
      )
    }

    const data = await response.json()

    // Decrypt vault key
    const vaultKey = await decryptVaultKeyAsync(
      data.encryptedVaultKey,
      data.salt,
      data.nonce,
      password,
    )

    // Cache decrypted vault key
    vaultKeyCache.value[vaultId] = {
      vaultKey,
      timestamp: Date.now(),
    }

    return vaultKey
  }

  /**
   * Pushes CRDT logs to the server
   */
  const pushLogsAsync = async (
    backendId: string,
    vaultId: string,
    logs: SelectHaexCrdtLogs[],
  ): Promise<void> => {
    const backend = syncBackendsStore.backends.find((b) => b.id === backendId)
    if (!backend) {
      throw new Error('Backend not found')
    }

    // Get vault key from cache
    const cached = vaultKeyCache.value[vaultId]
    if (!cached) {
      throw new Error('Vault key not available. Please unlock vault first.')
    }

    const vaultKey = cached.vaultKey

    // Get auth token
    const token = await getAuthTokenAsync()
    if (!token) {
      throw new Error('Not authenticated')
    }

    // Encrypt each log entry
    const encryptedLogs: SyncLogData[] = []
    for (const log of logs) {
      const { encryptedData, nonce } = await encryptCrdtDataAsync(
        log,
        vaultKey,
      )

      // Generate sequence number based on timestamp
      const sequence = Date.now()

      encryptedLogs.push({
        vaultId,
        encryptedData,
        nonce,
        haexTimestamp: log.haexTimestamp!,
        sequence,
      })
    }

    // Send to server
    const response = await fetch(`${backend.serverUrl}/sync/push`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({
        vaultId,
        logs: encryptedLogs,
      }),
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({}))
      throw new Error(
        `Failed to push logs: ${error.error || response.statusText}`,
      )
    }
  }

  /**
   * Pulls CRDT logs from the server
   */
  const pullLogsAsync = async (
    backendId: string,
    vaultId: string,
    afterSequence?: number,
    limit?: number,
  ): Promise<SelectHaexCrdtLogs[]> => {
    const backend = syncBackendsStore.backends.find((b) => b.id === backendId)
    if (!backend) {
      throw new Error('Backend not found')
    }

    // Get vault key from cache
    const cached = vaultKeyCache.value[vaultId]
    if (!cached) {
      throw new Error('Vault key not available. Please unlock vault first.')
    }

    const vaultKey = cached.vaultKey

    // Get auth token
    const token = await getAuthTokenAsync()
    if (!token) {
      throw new Error('Not authenticated')
    }

    // Fetch from server
    const response = await fetch(`${backend.serverUrl}/sync/pull`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({
        vaultId,
        afterSequence,
        limit: limit ?? 100,
      }),
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({}))
      throw new Error(
        `Failed to pull logs: ${error.error || response.statusText}`,
      )
    }

    const data: PullLogsResponse = await response.json()

    // Decrypt each log entry
    const decryptedLogs: SelectHaexCrdtLogs[] = []
    for (const log of data.logs) {
      try {
        const decrypted = await decryptCrdtDataAsync<SelectHaexCrdtLogs>(
          log.encryptedData,
          log.nonce,
          vaultKey,
        )
        decryptedLogs.push(decrypted)
      } catch (error) {
        console.error('Failed to decrypt log entry:', log.id, error)
        // Skip corrupted entries
      }
    }

    return decryptedLogs
  }

  /**
   * Clears vault key from cache
   */
  const clearVaultKeyCache = (vaultId?: string) => {
    if (vaultId) {
      delete vaultKeyCache.value[vaultId]
    } else {
      vaultKeyCache.value = {}
    }
  }

  /**
   * Health check - verifies server is reachable
   */
  const healthCheckAsync = async (backendId: string): Promise<boolean> => {
    const backend = syncBackendsStore.backends.find((b) => b.id === backendId)
    if (!backend) {
      return false
    }

    try {
      const response = await fetch(backend.serverUrl)
      return response.ok
    } catch {
      return false
    }
  }

  return {
    vaultKeyCache,
    supabaseClient,
    initSupabaseClientAsync,
    getAuthTokenAsync,
    storeVaultKeyAsync,
    getVaultKeyAsync,
    pushLogsAsync,
    pullLogsAsync,
    clearVaultKeyCache,
    healthCheckAsync,
  }
})

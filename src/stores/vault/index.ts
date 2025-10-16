// src/stores/vault/index.ts

import { drizzle } from 'drizzle-orm/sqlite-proxy'
import { invoke } from '@tauri-apps/api/core'
import { schema } from '@/../src-tauri/database/index'
import type {
  AsyncRemoteCallback,
  SqliteRemoteDatabase,
} from 'drizzle-orm/sqlite-proxy'

interface IVault {
  name: string
  drizzle: SqliteRemoteDatabase<typeof schema>
}
interface IOpenVaults {
  [vaultId: string]: IVault
}

export const useVaultStore = defineStore('vaultStore', () => {
  const {
    public: { haexVault },
  } = useRuntimeConfig()

  const currentVaultId = computed<string | undefined>({
    get: () =>
      getSingleRouteParam(useRouter().currentRoute.value.params.vaultId),
    set: (newVaultId) => {
      useRouter().currentRoute.value.params.vaultId = newVaultId ?? ''
    },
  })

  const currentVaultName = ref(haexVault.defaultVaultName || 'HaexHub')

  const openVaults = ref<IOpenVaults>({})

  const currentVault = computed(
    () => openVaults.value?.[currentVaultId.value ?? ''],
  )

  const openAsync = async ({
    path = '',
    password,
  }: {
    path: string
    password: string
  }) => {
    try {
      await invoke<string>('open_encrypted_database', {
        vaultPath: path,
        key: password,
      })

      const vaultId = await getVaultIdAsync(path)

      const fileName = getFileName(path) ?? path

      openVaults.value = {
        ...openVaults.value,
        [vaultId]: {
          name: fileName,
          drizzle: drizzle<typeof schema>(drizzleCallback, {
            schema: schema,
            logger: false,
          }),
        },
      }

      return vaultId
    } catch (error) {
      console.error('Error openAsync ', error)
      throw error
    }
  }

  const createAsync = async ({
    vaultName,
    password,
  }: {
    vaultName: string
    password: string
  }) => {
    const vaultPath = await invoke<string>('create_encrypted_database', {
      vaultName,
      key: password,
    })
    return await openAsync({ path: vaultPath, password })
  }

  const closeAsync = async () => {
    if (!currentVaultId.value) return

    delete openVaults.value?.[currentVaultId.value]
  }

  return {
    closeAsync,
    createAsync,
    currentVault,
    currentVaultId,
    currentVaultName,
    openAsync,
    openVaults,
  }
})

const getVaultIdAsync = async (path: string) => {
  const encoder = new TextEncoder()
  const data = encoder.encode(path)

  const hashBuffer = await crypto.subtle.digest('SHA-256', data)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('')
  return hashHex
}

const isSelectQuery = (sql: string) => {
  const selectRegex = /^\s*SELECT\b/i
  return selectRegex.test(sql)
}

const drizzleCallback = (async (
  sql: string,
  params: unknown[],
  method: 'get' | 'run' | 'all' | 'values',
) => {
  let rows: unknown[] = []

  if (isSelectQuery(sql)) {
    rows = await invoke<unknown[]>('sql_select', { sql, params }).catch((e) => {
      console.error('SQL select Error:', e, sql, params)
      return []
    })
  } else {
    rows = await invoke<unknown[]>('sql_execute', { sql, params }).catch(
      (e) => {
        console.error('SQL execute Error:', e, sql, params)
        return []
      },
    )
  }

  if (method === 'get') {
    return { rows: rows.length > 0 ? [rows[0]] : [] }
  } else {
    return { rows }
  }
}) satisfies AsyncRemoteCallback

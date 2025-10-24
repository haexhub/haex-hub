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

  const existsVault = () => {
    if (!currentVault.value?.drizzle) {
      console.error('Kein Vault geöffnet')
      return
    }
  }

  return {
    closeAsync,
    createAsync,
    currentVault,
    currentVaultId,
    currentVaultName,
    existsVault,
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

const hasReturning = (sql: string) => {
  const returningRegex = /\bRETURNING\b/i
  return returningRegex.test(sql)
}

const drizzleCallback = (async (
  sql: string,
  params: unknown[],
  method: 'get' | 'run' | 'all' | 'values',
) => {
  // Wir MÜSSEN 'any[]' verwenden, um Drizzle's Typ zu erfüllen.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let rows: any[] = []

  try {
    if (isSelectQuery(sql)) {
      // SELECT statements
      rows = await invoke<unknown[]>('sql_select_with_crdt', {
        sql,
        params,
      }).catch((e) => {
        console.error('SQL select Error:', e, sql, params)
        return []
      })
    } else if (hasReturning(sql)) {
      // INSERT/UPDATE/DELETE with RETURNING → use query
      rows = await invoke<unknown[]>('sql_query_with_crdt', {
        sql,
        params,
      }).catch((e) => {
        console.error('SQL query with CRDT Error:', e, sql, params)
        return []
      })
    } else {
      // INSERT/UPDATE/DELETE without RETURNING → use execute
      await invoke<unknown[]>('sql_execute_with_crdt', {
        sql,
        params,
      }).catch((e) => {
        console.error('SQL execute with CRDT Error:', e, sql, params, rows)
        return []
      })
    }
  } catch (error) {
    console.error('Fehler im drizzleCallback invoke:', error, {
      sql,
      params,
      method,
    })
  }

  console.log('drizzleCallback', method, sql, params)
  console.log('drizzleCallback rows', rows, rows.slice(0, 1))

  if (method === 'get') {
    return rows.length > 0 ? { rows: rows.at(0) } : { rows }
  }
  return { rows }
}) satisfies AsyncRemoteCallback

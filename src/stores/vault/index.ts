import { drizzle } from 'drizzle-orm/sqlite-proxy'
import { invoke } from '@tauri-apps/api/core'
import { platform } from '@tauri-apps/plugin-os'
import * as schema from '@/../src-tauri/database/schemas/vault'
import type { SqliteRemoteDatabase } from 'drizzle-orm/sqlite-proxy'
import { isTable, sql } from 'drizzle-orm'
import { getTableConfig } from 'drizzle-orm/sqlite-core'

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
      const result = await invoke<string>('open_encrypted_database', {
        path,
        key: password,
      })

      if (result !== 'success') throw new Error(result)

      const vaultId = await getVaultIdAsync(path)

      const fileName = getFileName(path)

      openVaults.value = {
        ...openVaults.value,
        [vaultId]: {
          name: fileName ?? path,
          drizzle: drizzle<typeof schema>(
            async (sql, params: unknown[], method) => {
              let rows: any[] = []
              let results: any[] = []

              // If the query is a SELECT, use the select method
              if (isSelectQuery(sql)) {
                console.log('sql_select', sql, params, method)
                rows = await invoke<unknown[]>('sql_select', {
                  sql,
                  params,
                }).catch((e) => {
                  console.error('SQL select Error:', e, sql, params)
                  return []
                })
              } else {
                console.log('sql_execute', sql, params, method)
                // Otherwise, use the execute method
                rows = await invoke<unknown[]>('sql_execute', {
                  sql,
                  params,
                }).catch((e) => {
                  console.error('SQL execute Error:', e, sql, params)
                  return []
                })
                return { rows: [] }
              }

              results = method === 'all' ? rows : rows[0]

              return { rows: results }
            },
            { schema: schema, logger: true },
          ),
        },
      }

      const { addVaultAsync } = useLastVaultStore()
      await addVaultAsync({ path })
      return vaultId
    } catch (error) {
      console.error('Error openAsync ', error)
      throw error
    }
  }

  const createAsync = async ({
    path,
    password,
  }: {
    path: string
    password: string
  }) => {
    await invoke('create_encrypted_database', {
      path,
      key: password,
    })
    return await openAsync({ path, password })
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
  console.log('check isSelectQuery', sql)
  const selectRegex = /^\s*SELECT\b/i
  return selectRegex.test(sql)
}

/* const trackAllHaexTablesAsync = async (vaultId: string) => {
  const { openVaults } = useVaultStore()
  const promises = Object.values(schema)
    .filter(isTable)
    .map((table) => {
      const stmt = `SELECT crsql_as_crr('${getTableConfig(table).name}');`
      console.log('track table', getTableConfig(table).name)

      return openVaults?.[vaultId]?.drizzle.run(sql`${stmt}`.getSQL())
    })

  await Promise.allSettled(promises)
} */

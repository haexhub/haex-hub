import * as schema from '@/../src-tauri/database/schemas/vault'
import { invoke } from '@tauri-apps/api/core'
import { platform } from '@tauri-apps/plugin-os'
import { eq } from 'drizzle-orm'
import type { SqliteRemoteDatabase } from 'drizzle-orm/sqlite-proxy'
import { drizzle } from 'drizzle-orm/sqlite-proxy'

interface IVault {
  name: string
  drizzle: SqliteRemoteDatabase<typeof schema>
}
interface IOpenVaults {
  [vaultId: string]: IVault
}

export const useVaultStore = defineStore('vaultStore', () => {
  const currentVaultId = computed<string | undefined>({
    get: () =>
      getSingleRouteParam(useRouter().currentRoute.value.params.vaultId),
    set: (newVaultId) => {
      useRouter().currentRoute.value.params.vaultId = newVaultId ?? ''
    },
  })

  const defaultVaultName = ref('HaexHub')
  const currentVaultName = ref(defaultVaultName.value)

  const read_only = computed<boolean>({
    get: () => {
      console.log(
        'query showSidebar',
        useRouter().currentRoute.value.query.readonly,
      )
      return JSON.parse(
        getSingleRouteParam(useRouter().currentRoute.value.query.readonly) ||
          'false',
      )
    },
    set: (readonly) => {
      const router = useRouter()
      router.replace({
        query: {
          ...router.currentRoute.value.query,
          readonly: JSON.stringify(readonly ? true : false),
        },
      })
    },
  })

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
      const seperator = platform() === 'windows' ? '\\' : '/'
      const fileName = path.split(seperator).pop()

      openVaults.value = {
        ...openVaults.value,
        [vaultId]: {
          name: fileName ?? path,
          drizzle: drizzle<typeof schema>(
            async (sql, params: unknown[], method) => {
              let rows: unknown[] = []
              let results: any = []

              // If the query is a SELECT, use the select method
              if (isSelectQuery(sql)) {
                console.log('sql_select', sql, params)
                rows = await invoke<unknown[]>('sql_select', {
                  sql,
                  params,
                }).catch((e) => {
                  console.error('SQL select Error:', e, sql, params)
                  return []
                })
                console.log('select', rows)
              } else {
                console.log('sql_execute', sql, params)
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
      throw new Error(JSON.stringify(error))
      return false
    }
  }

  const refreshDatabaseAsync = async () => {
    console.log('refreshDatabaseAsync')
    /*     if (!currentVault.value?.database.close) {
      return navigateTo(useLocaleRoute()({ name: 'vaultOpen' }));
    } */
  }

  const createAsync = async ({
    path,
    password,
  }: {
    path: string
    password: string
  }) => {
    /* const existDb = await exists('default.db', {
        baseDir: BaseDirectory.Resource,
      }); */

    /* const existDb = await resolveResource('resources/default.db');
    if (!existDb) throw new Error('Keine Datenbank da');
    await copyFile(existDb, path); */
    const result = await invoke('create_encrypted_database', {
      path,
      key: password,
    })
    console.log('create_encrypted_database', result)
    return await openAsync({ path, password })
  }

  const closeAsync = async () => {
    if (!currentVaultId.value) return

    /* if (
      typeof openVaults.value?.[currentVaultId.value]?.database?.close ===
      'function'
    ) {
      console.log('close db', openVaults.value?.[currentVaultId.value]);
      return openVaults.value?.[currentVaultId.value]?.database?.close();
    } */
    delete openVaults.value?.[currentVaultId.value]
  }

  const syncLocaleAsync = async () => {
    try {
      const app = useNuxtApp()

      const currentLocaleRow = await currentVault.value?.drizzle
        .select()
        .from(schema.haexSettings)
        .where(eq(schema.haexSettings.key, 'locale'))

      if (currentLocaleRow?.[0]?.value) {
        const currentLocale = app.$i18n.availableLocales.find(
          (locale) => locale === currentLocaleRow[0].value,
        )
        await app.$i18n.setLocale(currentLocale ?? app.$i18n.defaultLocale)
      } else {
        await currentVault.value?.drizzle.insert(schema.haexSettings).values({
          id: crypto.randomUUID(),
          key: 'locale',
          value: app.$i18n.locale.value,
        })
      }
    } catch (error) {
      console.log('ERROR syncLocaleAsync', error)
    }
  }

  const syncThemeAsync = async () => {
    const { availableThemes, defaultTheme, currentTheme } = storeToRefs(
      useUiStore(),
    )
    const currentThemeRow = await currentVault.value?.drizzle
      .select()
      .from(schema.haexSettings)
      .where(eq(schema.haexSettings.key, 'theme'))

    if (currentThemeRow?.[0]?.value) {
      const theme = availableThemes.value.find(
        (theme) => theme.name === currentThemeRow[0].value,
      )
      currentTheme.value = theme ?? defaultTheme.value
    } else {
      await currentVault.value?.drizzle.insert(schema.haexSettings).values({
        id: crypto.randomUUID(),
        key: 'theme',
        value: currentTheme.value.value,
      })
    }
  }

  const syncVaultNameAsync = async () => {
    const currentVaultNameRow = await currentVault.value?.drizzle
      .select()
      .from(schema.haexSettings)
      .where(eq(schema.haexSettings.key, 'vaultName'))

    if (currentVaultNameRow?.[0]?.value) {
      currentVaultName.value =
        currentVaultNameRow.at(0)?.value ?? defaultVaultName.value
    } else {
      await currentVault.value?.drizzle.insert(schema.haexSettings).values({
        id: crypto.randomUUID(),
        key: 'vaultName',
        value: currentVaultName.value,
      })
    }
  }

  const updateVaultNameAsync = async (newVaultName?: string | null) => {
    console.log('set new vaultName', newVaultName)
    return currentVault.value?.drizzle
      .update(schema.haexSettings)
      .set({ value: newVaultName ?? defaultVaultName.value })
      .where(eq(schema.haexSettings.key, 'vaultName'))
  }

  return {
    closeAsync,
    createAsync,
    currentVault,
    currentVaultId,
    currentVaultName,
    openAsync,
    openVaults,
    read_only,
    refreshDatabaseAsync,
    syncLocaleAsync,
    syncThemeAsync,
    syncVaultNameAsync,
    updateVaultNameAsync,
  }
})

const getVaultIdAsync = async (path: string) => {
  const encoder = new TextEncoder()
  const data = encoder.encode(path)

  const hashBuffer = await crypto.subtle.digest('SHA-256', data)
  const hashArray = Array.from(new Uint8Array(hashBuffer)) // convert buffer to byte array
  const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('') // convert bytes to hex string
  console.log('vaultId', hashHex)
  return hashHex
}

const isSelectQuery = (sql: string) => {
  const selectRegex = /^\s*SELECT\b/i
  return selectRegex.test(sql)
}

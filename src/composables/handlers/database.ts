import { invoke } from '@tauri-apps/api/core'
import { HAEXTENSION_METHODS } from '@haexhub/sdk'
import type { IHaexHubExtension } from '~/types/haexhub'
import type { ExtensionRequest } from './types'

export async function handleDatabaseMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  const params = request.params as {
    query?: string
    params?: unknown[]
  }

  switch (request.method) {
    case HAEXTENSION_METHODS.database.query: {
      try {
        const rows = await invoke<unknown[]>('extension_sql_select', {
          sql: params.query || '',
          params: params.params || [],
          publicKey: extension.publicKey,
          name: extension.name,
        })

        return {
          rows,
          rowsAffected: 0,
          lastInsertId: undefined,
        }
      } catch (error) {
        // If error is about non-SELECT statements (INSERT/UPDATE/DELETE with RETURNING),
        // automatically retry with execute
        if (error?.message?.includes('Only SELECT statements are allowed')) {
          const rows = await invoke<unknown[]>('extension_sql_execute', {
            sql: params.query || '',
            params: params.params || [],
            publicKey: extension.publicKey,
            name: extension.name,
          })

          return {
            rows,
            rowsAffected: rows.length,
            lastInsertId: undefined,
          }
        }
        throw error
      }
    }

    case HAEXTENSION_METHODS.database.execute: {
      const rows = await invoke<unknown[]>('extension_sql_execute', {
        sql: params.query || '',
        params: params.params || [],
        publicKey: extension.publicKey,
        name: extension.name,
      })

      return {
        rows,
        rowsAffected: 1,
        lastInsertId: undefined,
      }
    }

    case HAEXTENSION_METHODS.database.transaction: {
      const statements =
        (request.params as { statements?: string[] }).statements || []

      for (const stmt of statements) {
        await invoke('extension_sql_execute', {
          sql: stmt,
          params: [],
          publicKey: extension.publicKey,
          name: extension.name,
        })
      }

      return { success: true }
    }

    default:
      throw new Error(`Unknown database method: ${request.method}`)
  }
}

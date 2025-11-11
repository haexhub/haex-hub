import type { IHaexHubExtension } from '~/types/haexhub'
import type { ExtensionRequest } from './types'
import { invoke } from '@tauri-apps/api/core'

export async function handlePermissionsMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  if (!extension || !request) {
    throw new Error('Extension not found')
  }

  const { method, params } = request

  if (method === 'permissions.web.check') {
    return await checkWebPermissionAsync(params, extension)
  }

  if (method === 'permissions.database.check') {
    return await checkDatabasePermissionAsync(params, extension)
  }

  if (method === 'permissions.filesystem.check') {
    return await checkFilesystemPermissionAsync(params, extension)
  }

  throw new Error(`Unknown permission method: ${method}`)
}

async function checkWebPermissionAsync(
  params: Record<string, unknown>,
  extension: IHaexHubExtension,
) {
  const url = params.url as string
  const method = (params.method as string) || 'GET'

  if (!url) {
    throw new Error('URL is required')
  }

  try {
    await invoke<void>('check_web_permission', {
      extensionId: extension.id,
      method,
      url,
    })

    return { status: 'granted' }
  } catch (error: any) {
    // Permission denied errors return a specific error code
    if (error?.code === 1002 || error?.message?.includes('Permission denied')) {
      return { status: 'denied' }
    }
    // Other errors should be thrown
    throw error
  }
}

async function checkDatabasePermissionAsync(
  params: Record<string, unknown>,
  extension: IHaexHubExtension,
) {
  const resource = params.resource as string
  const operation = params.operation as string

  if (!resource || !operation) {
    throw new Error('Resource and operation are required')
  }

  try {
    await invoke<void>('check_database_permission', {
      extensionId: extension.id,
      resource,
      operation,
    })

    return { status: 'granted' }
  } catch (error: any) {
    if (error?.code === 1002 || error?.message?.includes('Permission denied')) {
      return { status: 'denied' }
    }
    throw error
  }
}

async function checkFilesystemPermissionAsync(
  params: Record<string, unknown>,
  extension: IHaexHubExtension,
) {
  const path = params.path as string
  const operation = params.operation as string

  if (!path || !operation) {
    throw new Error('Path and operation are required')
  }

  try {
    await invoke<void>('check_filesystem_permission', {
      extensionId: extension.id,
      path,
      operation,
    })

    return { status: 'granted' }
  } catch (error: any) {
    if (error?.code === 1002 || error?.message?.includes('Permission denied')) {
      return { status: 'denied' }
    }
    throw error
  }
}

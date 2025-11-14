import type { IHaexHubExtension } from '~/types/haexhub'
import type { ExtensionRequest } from './types'
import { invoke } from '@tauri-apps/api/core'
import { HAEXTENSION_METHODS } from '@haexhub/sdk'

export async function handleWebMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  if (!extension || !request) {
    throw new Error('Extension not found')
  }

  const { method, params } = request

  if (method === HAEXTENSION_METHODS.web.fetch) {
    return await handleWebFetchAsync(params, extension)
  }

  if (method === HAEXTENSION_METHODS.application.open) {
    return await handleWebOpenAsync(params, extension)
  }

  throw new Error(`Unknown web method: ${method}`)
}

async function handleWebFetchAsync(
  params: Record<string, unknown>,
  extension: IHaexHubExtension,
) {
  const url = params.url as string
  const method = (params.method as string) || undefined
  const headers = (params.headers as Record<string, string>) || undefined
  const body = params.body as string | undefined
  const timeout = (params.timeout as number) || undefined

  if (!url) {
    throw new Error('URL is required')
  }

  try {
    // Call Rust backend through Tauri IPC to avoid CORS restrictions
    const response = await invoke<{
      status: number
      status_text: string
      headers: Record<string, string>
      body: string
      url: string
    }>('extension_web_fetch', {
      url,
      method,
      headers,
      body,
      timeout,
      publicKey: extension.publicKey,
      name: extension.name,
    })

    return {
      status: response.status,
      statusText: response.status_text,
      headers: response.headers,
      body: response.body,
      url: response.url,
    }
  } catch (error: any) {
    console.error('Web request error:', error)

    // Check if it's a permission denied error
    if (error?.code === 1002 || error?.message?.includes('Permission denied')) {
      const toast = useToast()
      toast.add({
        title: 'Permission denied',
        description: `Extension "${extension.name}" does not have permission to access ${url}`,
        color: 'error',
      })
    }

    if (error instanceof Error) {
      throw new Error(`Web request failed: ${error.message}`)
    }
    throw new Error(`Web request failed with unknown error: ${JSON.stringify(error)}`)
  }
}

async function handleWebOpenAsync(
  params: Record<string, unknown>,
  extension: IHaexHubExtension,
) {
  const url = params.url as string

  if (!url) {
    throw new Error('URL is required')
  }

  try {
    // Call Rust backend to open URL in default browser
    await invoke<void>('extension_web_open', {
      url,
      publicKey: extension.publicKey,
      name: extension.name,
    })
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`Failed to open URL: ${error.message}`)
    }
    throw new Error('Failed to open URL with unknown error')
  }
}

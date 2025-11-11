import type { IHaexHubExtension } from '~/types/haexhub'
import type { ExtensionRequest } from './types'

export async function handleWebMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  if (!extension || !request) {
    throw new Error('Extension not found')
  }

  // TODO: Add permission check for web requests
  // This should verify that the extension has permission to make web requests
  // before proceeding with the fetch operation

  const { method, params } = request

  if (method === 'haextension.web.fetch') {
    return await handleWebFetchAsync(params)
  }

  throw new Error(`Unknown web method: ${method}`)
}

async function handleWebFetchAsync(params: Record<string, unknown>) {
  const url = params.url as string
  const method = (params.method as string) || 'GET'
  const headers = (params.headers as Record<string, string>) || {}
  const body = params.body as string | undefined
  const timeout = (params.timeout as number) || 30000

  if (!url) {
    throw new Error('URL is required')
  }

  try {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), timeout)

    const fetchOptions: RequestInit = {
      method,
      headers,
      signal: controller.signal,
    }

    // Convert base64 body back to binary if present
    if (body) {
      const binaryString = atob(body)
      const bytes = new Uint8Array(binaryString.length)
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i)
      }
      fetchOptions.body = bytes
    }

    const response = await fetch(url, fetchOptions)
    clearTimeout(timeoutId)

    // Read response as ArrayBuffer
    const responseBody = await response.arrayBuffer()

    // Convert ArrayBuffer to base64
    const bytes = new Uint8Array(responseBody)
    let binary = ''
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i])
    }
    const base64Body = btoa(binary)

    // Convert headers to plain object
    const responseHeaders: Record<string, string> = {}
    response.headers.forEach((value, key) => {
      responseHeaders[key] = value
    })

    return {
      status: response.status,
      statusText: response.statusText,
      headers: responseHeaders,
      body: base64Body,
      url: response.url,
    }
  } catch (error) {
    if (error instanceof Error) {
      if (error.name === 'AbortError') {
        throw new Error(`Request timeout after ${timeout}ms`)
      }
      throw new Error(`Fetch failed: ${error.message}`)
    }
    throw new Error('Fetch failed with unknown error')
  }
}

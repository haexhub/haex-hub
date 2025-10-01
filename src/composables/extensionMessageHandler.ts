// composables/extensionMessageHandler.ts
import { invoke } from '@tauri-apps/api/core'
import type { IHaexHubExtension } from '~/types/haexhub'

interface ExtensionRequest {
  id: string
  method: string
  params: Record<string, unknown>
  timestamp: number
}

// Globaler Handler - nur einmal registriert
let globalHandlerRegistered = false
const iframeRegistry = new Map<HTMLIFrameElement, IHaexHubExtension>()

const registerGlobalMessageHandler = () => {
  if (globalHandlerRegistered) return

  window.addEventListener('message', async (event: MessageEvent) => {
    // Finde die Extension für dieses IFrame
    let extension: IHaexHubExtension | undefined
    let sourceIframe: HTMLIFrameElement | undefined

    for (const [iframe, ext] of iframeRegistry.entries()) {
      if (event.source === iframe.contentWindow) {
        extension = ext
        sourceIframe = iframe
        break
      }
    }

    if (!extension || !sourceIframe) {
      return // Message ist nicht von einem registrierten IFrame
    }

    const request = event.data as ExtensionRequest

    if (!request.id || !request.method) {
      console.error('Invalid extension request:', request)
      return
    }

    console.log(
      `[HaexHub] ${extension.name} request:`,
      request.method,
      request.params,
    )

    try {
      let result: unknown

      if (request.method.startsWith('extension.')) {
        result = await handleExtensionMethodAsync(request, extension)
      } else if (request.method.startsWith('db.')) {
        result = await handleDatabaseMethodAsync(request, extension)
      } else if (request.method.startsWith('fs.')) {
        result = await handleFilesystemMethodAsync(request, extension)
      } else if (request.method.startsWith('http.')) {
        result = await handleHttpMethodAsync(request, extension)
      } else if (request.method.startsWith('permissions.')) {
        result = await handlePermissionsMethodAsync(request, extension)
      } else if (request.method.startsWith('context.')) {
        result = await handleContextMethodAsync(request)
      } else {
        throw new Error(`Unknown method: ${request.method}`)
      }

      sourceIframe.contentWindow?.postMessage(
        {
          id: request.id,
          result,
        },
        '*',
      )
    } catch (error) {
      console.error('[HaexHub] Extension request error:', error)

      sourceIframe.contentWindow?.postMessage(
        {
          id: request.id,
          error: {
            code: 'INTERNAL_ERROR',
            message: error instanceof Error ? error.message : 'Unknown error',
            details: error,
          },
        },
        '*',
      )
    }
  })

  globalHandlerRegistered = true
}

export const useExtensionMessageHandler = (
  iframeRef: Ref<HTMLIFrameElement | undefined | null>,
  extension: ComputedRef<IHaexHubExtension | undefined | null>,
) => {
  // Registriere globalen Handler beim ersten Aufruf
  registerGlobalMessageHandler()

  // Registriere dieses IFrame
  watchEffect(() => {
    if (iframeRef.value && extension.value) {
      iframeRegistry.set(iframeRef.value, extension.value)
    }
  })

  // Cleanup beim Unmount
  onUnmounted(() => {
    if (iframeRef.value) {
      iframeRegistry.delete(iframeRef.value)
    }
  })
}

// ==========================================
// Extension Methods
// ==========================================

async function handleExtensionMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension, // Direkter Typ, kein ComputedRef mehr
) {
  switch (request.method) {
    case 'extension.getInfo':
      return await invoke('get_extension_info', {
        extensionId: extension.id,
      })
    default:
      throw new Error(`Unknown extension method: ${request.method}`)
  }
}

// ==========================================
// Database Methods
// ==========================================

async function handleDatabaseMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension, // Direkter Typ
) {
  const params = request.params as {
    query?: string
    params?: unknown[]
  }

  switch (request.method) {
    case 'db.query': {
      const rows = await invoke<unknown[]>('extension_sql_select', {
        sql: params.query || '',
        params: params.params || [],
        extensionId: extension.id,
      })

      return {
        rows,
        rowsAffected: 0,
        lastInsertId: undefined,
      }
    }

    case 'db.execute': {
      await invoke<string[]>('extension_sql_execute', {
        sql: params.query || '',
        params: params.params || [],
        extensionId: extension.id,
      })

      return {
        rows: [],
        rowsAffected: 1,
        lastInsertId: undefined,
      }
    }

    case 'db.transaction': {
      const statements =
        (request.params as { statements?: string[] }).statements || []

      for (const stmt of statements) {
        await invoke('extension_sql_execute', {
          sql: stmt,
          params: [],
          extensionId: extension.id,
        })
      }

      return { success: true }
    }

    default:
      throw new Error(`Unknown database method: ${request.method}`)
  }
}
// ==========================================
// Filesystem Methods (TODO)
// ==========================================

async function handleFilesystemMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  if (!request || !extension) return
  // TODO: Implementiere Filesystem Commands im Backend
  throw new Error('Filesystem methods not yet implemented')
}

// ==========================================
// HTTP Methods (TODO)
// ==========================================

async function handleHttpMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  if (!extension || !request) {
    throw new Error('Extension not found')
  }

  // TODO: Implementiere HTTP Commands im Backend
  throw new Error('HTTP methods not yet implemented')
}

// ==========================================
// Permission Methods (TODO)
// ==========================================

async function handlePermissionsMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  if (!extension || !request) {
    throw new Error('Extension not found')
  }

  // TODO: Implementiere Permission Request UI
  throw new Error('Permission methods not yet implemented')
}

// ==========================================
// Context Methods
// ==========================================

async function handleContextMethodAsync(request: ExtensionRequest) {
  const { currentTheme } = storeToRefs(useUiStore())
  const { locale } = useI18n()

  switch (request.method) {
    case 'context.get':
      return {
        theme: currentTheme.value || 'system',
        locale: locale.value,
        platform: detectPlatform(),
      }

    default:
      throw new Error(`Unknown context method: ${request.method}`)
  }
}

function detectPlatform(): 'desktop' | 'mobile' | 'tablet' {
  const width = window.innerWidth
  if (width < 768) return 'mobile'
  if (width < 1024) return 'tablet'
  return 'desktop'
}

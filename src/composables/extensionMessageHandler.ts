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

// Store context values that need to be accessed outside setup
let contextGetters: {
  getTheme: () => string
  getLocale: () => string
} | null = null

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
      } else if (request.method.startsWith('storage.')) {
        result = await handleStorageMethodAsync(request, extension)
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
  // Initialize context getters (can use composables here because we're in setup)
  const { currentTheme } = storeToRefs(useUiStore())
  const { locale } = useI18n()

  // Store getters for use outside setup context
  if (!contextGetters) {
    contextGetters = {
      getTheme: () => currentTheme.value?.value || 'system',
      getLocale: () => locale.value,
    }
  }

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

// Export Funktion für manuelle IFrame-Registrierung (kein Composable!)
export const registerExtensionIFrame = (
  iframe: HTMLIFrameElement,
  extension: IHaexHubExtension,
) => {
  // Stelle sicher, dass der globale Handler registriert ist
  registerGlobalMessageHandler()

  // Warnung wenn Context Getters nicht initialisiert wurden
  if (!contextGetters) {
    console.warn(
      'Context getters not initialized. Make sure useExtensionMessageHandler was called in setup context first.',
    )
  }

  iframeRegistry.set(iframe, extension)
}

export const unregisterExtensionIFrame = (iframe: HTMLIFrameElement) => {
  iframeRegistry.delete(iframe)
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
  switch (request.method) {
    case 'context.get':
      if (!contextGetters) {
        throw new Error(
          'Context not initialized. Make sure useExtensionMessageHandler is called in a component.',
        )
      }
      return {
        theme: contextGetters.getTheme(),
        locale: contextGetters.getLocale(),
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

// ==========================================
// Storage Methods
// ==========================================

async function handleStorageMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  const storageKey = `ext_${extension.id}_`
  console.log(`[HaexHub Storage] ${request.method} for extension ${extension.id}`)

  switch (request.method) {
    case 'storage.getItem': {
      const key = request.params.key as string
      return localStorage.getItem(storageKey + key)
    }

    case 'storage.setItem': {
      const key = request.params.key as string
      const value = request.params.value as string
      localStorage.setItem(storageKey + key, value)
      return null
    }

    case 'storage.removeItem': {
      const key = request.params.key as string
      localStorage.removeItem(storageKey + key)
      return null
    }

    case 'storage.clear': {
      // Remove only extension-specific keys
      const keys = Object.keys(localStorage).filter(k => k.startsWith(storageKey))
      keys.forEach(k => localStorage.removeItem(k))
      return null
    }

    case 'storage.keys': {
      // Return only extension-specific keys (without prefix)
      const keys = Object.keys(localStorage)
        .filter(k => k.startsWith(storageKey))
        .map(k => k.substring(storageKey.length))
      return keys
    }

    default:
      throw new Error(`Unknown storage method: ${request.method}`)
  }
}

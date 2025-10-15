// composables/extensionMessageHandler.ts
import { invoke } from '@tauri-apps/api/core'
import type { IHaexHubExtension } from '~/types/haexhub'
import {
  EXTENSION_PROTOCOL_NAME,
  EXTENSION_PROTOCOL_PREFIX,
} from '~/config/constants'
import type { Platform } from '@tauri-apps/plugin-os'

interface ExtensionRequest {
  id: string
  method: string
  params: Record<string, unknown>
  timestamp: number
}

// Globaler Handler - nur einmal registriert
let globalHandlerRegistered = false
const iframeRegistry = new Map<HTMLIFrameElement, IHaexHubExtension>()
// Map event.source (WindowProxy) to extension for sandbox-compatible matching
const sourceRegistry = new Map<Window, IHaexHubExtension>()
// Reverse map: extension ID to Window for broadcasting
const extensionToWindowMap = new Map<string, Window>()

// Store context values that need to be accessed outside setup
let contextGetters: {
  getTheme: () => string
  getLocale: () => string
  getPlatform: () => Platform | undefined
} | null = null

const registerGlobalMessageHandler = () => {
  if (globalHandlerRegistered) return

  window.addEventListener('message', async (event: MessageEvent) => {
    // Ignore console.forward messages - they're handled elsewhere
    if (event.data?.type === 'console.forward') {
      return
    }

    const request = event.data as ExtensionRequest

    // Find extension by decoding event.origin (works with sandboxed iframes)
    // Origin formats:
    // - Desktop: haex-extension://<base64>
    // - Android: http://haex-extension.localhost (need to check request URL for base64)
    let extension: IHaexHubExtension | undefined

    console.log(
      '[ExtensionHandler] Received message from origin:',
      event.origin,
    )

    // Try to decode extension info from origin
    if (event.origin) {
      let base64Host: string | null = null

      if (event.origin.startsWith(EXTENSION_PROTOCOL_PREFIX)) {
        // Desktop format: haex-extension://<base64>
        base64Host = event.origin.replace(EXTENSION_PROTOCOL_PREFIX, '')
        console.log(
          '[ExtensionHandler] Extracted base64 (custom protocol):',
          base64Host,
        )
      } else if (
        event.origin === `http://${EXTENSION_PROTOCOL_NAME}.localhost`
      ) {
        // Android format: http://haex-extension.localhost/{base64} (origin doesn't contain extension info)
        // We need to identify extension by iframe source or fallback to single-extension mode
        console.log(
          `[ExtensionHandler] Android format detected (http://${EXTENSION_PROTOCOL_NAME}.localhost)`,
        )
        // Fallback to single iframe mode
        if (iframeRegistry.size === 1) {
          const entry = Array.from(iframeRegistry.entries())[0]
          if (entry) {
            const [_, ext] = entry
            extension = ext
            sourceRegistry.set(event.source as Window, ext)
            extensionToWindowMap.set(ext.id, event.source as Window)
          }
        }
      }

      if (base64Host && base64Host !== 'localhost') {
        try {
          const decodedInfo = JSON.parse(atob(base64Host)) as {
            name: string
            publicKey: string
            version: string
          }

          // Find matching extension in registry
          for (const [_, ext] of iframeRegistry.entries()) {
            if (
              ext.name === decodedInfo.name &&
              ext.publicKey === decodedInfo.publicKey &&
              ext.version === decodedInfo.version
            ) {
              extension = ext
              // Register for future lookups
              sourceRegistry.set(event.source as Window, ext)
              extensionToWindowMap.set(ext.id, event.source as Window)
              break
            }
          }
        } catch (e) {
          console.error('[ExtensionHandler] Failed to decode origin:', e)
        }
      }
    }

    // Fallback: Try to find extension by event.source (for localhost origin or legacy)
    if (!extension) {
      extension = sourceRegistry.get(event.source as Window)

      // If not registered yet, register on first message from this source
      if (!extension && iframeRegistry.size === 1) {
        // If we only have one iframe, assume this message is from it
        const entry = Array.from(iframeRegistry.entries())[0]
        if (entry) {
          const [_, ext] = entry
          const windowSource = event.source as Window
          sourceRegistry.set(windowSource, ext)
          extensionToWindowMap.set(ext.id, windowSource)
          extension = ext
        }
      } else if (extension && !extensionToWindowMap.has(extension.id)) {
        // Also register in reverse map for broadcasting
        extensionToWindowMap.set(extension.id, event.source as Window)
      }
    }

    if (!extension) {
      console.warn(
        '[ExtensionHandler] Could not identify extension for message:',
        event.origin,
      )
      return // Message ist nicht von einem registrierten IFrame
    }

    if (!request.id || !request.method) {
      console.error('[ExtensionHandler] Invalid extension request:', request)
      return
    }

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

      // Use event.source instead of contentWindow to work with sandboxed iframes
      // For sandboxed iframes, event.origin is "null" (string), which is not valid for postMessage
      const targetOrigin = event.origin === 'null' ? '*' : event.origin || '*'

      ;(event.source as Window)?.postMessage(
        {
          id: request.id,
          result,
        },
        targetOrigin,
      )
    } catch (error) {
      console.error('[ExtensionHandler] Extension request error:', error)

      // Use event.source instead of contentWindow to work with sandboxed iframes
      // For sandboxed iframes, event.origin is "null" (string), which is not valid for postMessage
      const targetOrigin = event.origin === 'null' ? '*' : event.origin || '*'

      ;(event.source as Window)?.postMessage(
        {
          id: request.id,
          error: {
            code: 'INTERNAL_ERROR',
            message: error instanceof Error ? error.message : 'Unknown error',
            details: error,
          },
        },
        targetOrigin,
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
  const { platform } = useDeviceStore()
  // Store getters for use outside setup context
  if (!contextGetters) {
    contextGetters = {
      getTheme: () => currentTheme.value?.value || 'system',
      getLocale: () => locale.value,
      getPlatform: () => platform,
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
  // Also remove from source registry
  const ext = iframeRegistry.get(iframe)
  if (ext) {
    // Find and remove all sources pointing to this extension
    for (const [source, extension] of sourceRegistry.entries()) {
      if (extension === ext) {
        sourceRegistry.delete(source)
      }
    }
    // Remove from extension-to-window map
    extensionToWindowMap.delete(ext.id)
  }
  iframeRegistry.delete(iframe)
}

// Export function to get Window for an extension (for broadcasting)
export const getExtensionWindow = (extensionId: string): Window | undefined => {
  return extensionToWindowMap.get(extensionId)
}

// ==========================================
// Extension Methods
// ==========================================

async function handleExtensionMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension, // Direkter Typ, kein ComputedRef mehr
) {
  switch (request.method) {
    case 'extension.getInfo': {
      const info = (await invoke('get_extension_info', {
        publicKey: extension.publicKey,
        name: extension.name,
      })) as Record<string, unknown>
      // Override allowedOrigin with the actual window origin
      // This fixes the dev-mode issue where Rust returns "tauri://localhost"
      // but the actual origin is "http://localhost:3003"
      return {
        ...info,
        allowedOrigin: window.location.origin,
      }
    }
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
  console.log(
    `[HaexHub Storage] ${request.method} for extension ${extension.id}`,
  )

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
      const keys = Object.keys(localStorage).filter((k) =>
        k.startsWith(storageKey),
      )
      keys.forEach((k) => localStorage.removeItem(k))
      return null
    }

    case 'storage.keys': {
      // Return only extension-specific keys (without prefix)
      const keys = Object.keys(localStorage)
        .filter((k) => k.startsWith(storageKey))
        .map((k) => k.substring(storageKey.length))
      return keys
    }

    default:
      throw new Error(`Unknown storage method: ${request.method}`)
  }
}

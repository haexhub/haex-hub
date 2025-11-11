// composables/extensionMessageHandler.ts
import type { IHaexHubExtension } from '~/types/haexhub'
import {
  EXTENSION_PROTOCOL_NAME,
  EXTENSION_PROTOCOL_PREFIX,
} from '~/config/constants'
import {
  handleDatabaseMethodAsync,
  handleFilesystemMethodAsync,
  handleWebMethodAsync,
  handlePermissionsMethodAsync,
  handleContextMethodAsync,
  handleStorageMethodAsync,
  setContextGetters,
  type ExtensionRequest,
  type ExtensionInstance,
} from './handlers'

// Globaler Handler - nur einmal registriert
let globalHandlerRegistered = false
const iframeRegistry = new Map<HTMLIFrameElement, ExtensionInstance>()
// Map event.source (WindowProxy) to extension instance for sandbox-compatible matching
const sourceRegistry = new Map<Window, ExtensionInstance>()
// Reverse map: window ID to Window for broadcasting (supports multiple windows per extension)
const windowIdToWindowMap = new Map<string, Window>()

const registerGlobalMessageHandler = () => {
  if (globalHandlerRegistered) return

  window.addEventListener('message', async (event: MessageEvent) => {
    // Ignore console.forward messages - they're handled elsewhere
    if (event.data?.type === 'console.forward') {
      return
    }

    const request = event.data as ExtensionRequest

    // Find extension instance by decoding event.origin (works with sandboxed iframes)
    // Origin formats:
    // - Desktop: haex-extension://<base64>
    // - Android: http://haex-extension.localhost (need to check request URL for base64)
    let instance: ExtensionInstance | undefined

    // Debug: Find which extension sent this message
    let sourceInfo = 'unknown source'
    for (const [iframe, inst] of iframeRegistry.entries()) {
      if (iframe.contentWindow === event.source) {
        sourceInfo = `${inst.extension.name} (${inst.windowId})`
        break
      }
    }
    console.log(
      '[ExtensionHandler] Received message from:',
      sourceInfo,
      'Method:',
      request.method,
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
            const [_, inst] = entry
            instance = inst
            sourceRegistry.set(event.source as Window, inst)
            windowIdToWindowMap.set(inst.windowId, event.source as Window)
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
          for (const [_, inst] of iframeRegistry.entries()) {
            if (
              inst.extension.name === decodedInfo.name &&
              inst.extension.publicKey === decodedInfo.publicKey &&
              inst.extension.version === decodedInfo.version
            ) {
              instance = inst
              // Register for future lookups
              sourceRegistry.set(event.source as Window, inst)
              windowIdToWindowMap.set(inst.windowId, event.source as Window)
              break
            }
          }
        } catch (e) {
          console.error('[ExtensionHandler] Failed to decode origin:', e)
        }
      }
    }

    // Fallback: Try to find extension instance by event.source (for localhost origin or legacy)
    if (!instance) {
      instance = sourceRegistry.get(event.source as Window)

      // If not registered yet, find by matching iframe.contentWindow to event.source
      if (!instance) {
        for (const [iframe, inst] of iframeRegistry.entries()) {
          if (iframe.contentWindow === event.source) {
            instance = inst
            // Register for future lookups
            sourceRegistry.set(event.source as Window, inst)
            windowIdToWindowMap.set(inst.windowId, event.source as Window)
            console.log(
              '[ExtensionHandler] Registered instance via contentWindow match:',
              inst.windowId,
            )
            break
          }
        }
      } else if (instance && !windowIdToWindowMap.has(instance.windowId)) {
        // Also register in reverse map for broadcasting
        windowIdToWindowMap.set(instance.windowId, event.source as Window)
      }
    }

    if (!instance) {
      console.warn(
        '[ExtensionHandler] Could not identify extension instance from event.source.',
        'Registered iframes:',
        iframeRegistry.size,
      )
      return // Message ist nicht von einem registrierten IFrame
    }

    if (!request.id || !request.method) {
      console.error('[ExtensionHandler] Invalid extension request:', request)
      return
    }

    try {
      let result: unknown

      if (request.method.startsWith('haextension.context.')) {
        result = await handleContextMethodAsync(request)
      } else if (request.method.startsWith('haextension.storage.')) {
        result = await handleStorageMethodAsync(request, instance)
      } else if (request.method.startsWith('haextension.db.')) {
        result = await handleDatabaseMethodAsync(request, instance.extension)
      } else if (request.method.startsWith('haextension.fs.')) {
        result = await handleFilesystemMethodAsync(request, instance.extension)
      } else if (request.method.startsWith('haextension.web.')) {
        result = await handleWebMethodAsync(request, instance.extension)
      } else if (request.method.startsWith('haextension.permissions.')) {
        result = await handlePermissionsMethodAsync(request, instance.extension)
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
  windowId: Ref<string>,
) => {
  // Initialize context getters (can use composables here because we're in setup)
  const { currentTheme } = storeToRefs(useUiStore())
  const { locale } = useI18n()
  const { platform } = useDeviceStore()
  // Store getters for use outside setup context
  setContextGetters({
    getTheme: () => currentTheme.value?.value || 'system',
    getLocale: () => locale.value,
    getPlatform: () => platform,
  })

  // Registriere globalen Handler beim ersten Aufruf
  registerGlobalMessageHandler()

  // Registriere dieses IFrame
  watchEffect(() => {
    if (iframeRef.value && extension.value) {
      iframeRegistry.set(iframeRef.value, {
        extension: extension.value,
        windowId: windowId.value,
      })
    }
  })

  // Cleanup beim Unmount
  onUnmounted(() => {
    if (iframeRef.value) {
      const instance = iframeRegistry.get(iframeRef.value)
      if (instance) {
        // Remove from all maps
        windowIdToWindowMap.delete(instance.windowId)
        for (const [source, inst] of sourceRegistry.entries()) {
          if (inst.windowId === instance.windowId) {
            sourceRegistry.delete(source)
          }
        }
      }
      iframeRegistry.delete(iframeRef.value)
    }
  })
}

// Export Funktion für manuelle IFrame-Registrierung (kein Composable!)
export const registerExtensionIFrame = (
  iframe: HTMLIFrameElement,
  extension: IHaexHubExtension,
  windowId: string,
) => {
  // Stelle sicher, dass der globale Handler registriert ist
  registerGlobalMessageHandler()

  // Note: Context getters should be initialized via useExtensionMessageHandler first

  iframeRegistry.set(iframe, { extension, windowId })
}

export const unregisterExtensionIFrame = (iframe: HTMLIFrameElement) => {
  // Also remove from source registry and instance map
  const instance = iframeRegistry.get(iframe)
  if (instance) {
    // Find and remove all sources pointing to this instance
    for (const [source, inst] of sourceRegistry.entries()) {
      if (inst.windowId === instance.windowId) {
        sourceRegistry.delete(source)
      }
    }
    // Remove from instance-to-window map
    windowIdToWindowMap.delete(instance.windowId)
  }
  iframeRegistry.delete(iframe)
}

// Export function to get Window for a specific instance (for broadcasting)
export const getInstanceWindow = (windowId: string): Window | undefined => {
  return windowIdToWindowMap.get(windowId)
}

// Get all windows for an extension (all instances)
export const getAllInstanceWindows = (extensionId: string): Window[] => {
  const windows: Window[] = []
  for (const [_, instance] of iframeRegistry.entries()) {
    if (instance.extension.id === extensionId) {
      const win = windowIdToWindowMap.get(instance.windowId)
      if (win) {
        windows.push(win)
      }
    }
  }
  return windows
}

// Deprecated - kept for backwards compatibility
export const getExtensionWindow = (extensionId: string): Window | undefined => {
  // Return first window for this extension
  return getAllInstanceWindows(extensionId)[0]
}

// Broadcast context changes to all extension instances
export const broadcastContextToAllExtensions = (context: {
  theme: string
  locale: string
  platform?: string
}) => {
  const message = {
    type: 'haextension.context.changed',
    data: { context },
    timestamp: Date.now(),
  }

  console.log(
    '[ExtensionHandler] Broadcasting context to all extensions:',
    context,
  )

  // Send to all registered extension windows
  for (const [_, instance] of iframeRegistry.entries()) {
    const win = windowIdToWindowMap.get(instance.windowId)
    if (win) {
      console.log(
        '[ExtensionHandler] Sending context to:',
        instance.extension.name,
        instance.windowId,
      )
      win.postMessage(message, '*')
    }
  }
}

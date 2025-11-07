import type { ExtensionRequest, ExtensionInstance } from './types'

export async function handleStorageMethodAsync(
  request: ExtensionRequest,
  instance: ExtensionInstance,
) {
  // Storage is now per-window, not per-extension
  const storageKey = `ext_${instance.extension.id}_${instance.windowId}_`
  console.log(
    `[HaexHub Storage] ${request.method} for window ${instance.windowId}`,
  )

  switch (request.method) {
    case 'haextension.storage.getItem': {
      const key = request.params.key as string
      return localStorage.getItem(storageKey + key)
    }

    case 'haextension.storage.setItem': {
      const key = request.params.key as string
      const value = request.params.value as string
      localStorage.setItem(storageKey + key, value)
      return null
    }

    case 'haextension.storage.removeItem': {
      const key = request.params.key as string
      localStorage.removeItem(storageKey + key)
      return null
    }

    case 'haextension.storage.clear': {
      // Remove only instance-specific keys
      const keys = Object.keys(localStorage).filter((k) =>
        k.startsWith(storageKey),
      )
      keys.forEach((k) => localStorage.removeItem(k))
      return null
    }

    case 'haextension.storage.keys': {
      // Return only instance-specific keys (without prefix)
      const keys = Object.keys(localStorage)
        .filter((k) => k.startsWith(storageKey))
        .map((k) => k.substring(storageKey.length))
      return keys
    }

    default:
      throw new Error(`Unknown storage method: ${request.method}`)
  }
}

import type { Platform } from '@tauri-apps/plugin-os'
import { HAEXTENSION_METHODS } from '@haexhub/sdk'
import type { ExtensionRequest } from './types'

// Context getters are set from the main handler during initialization
let contextGetters: {
  getTheme: () => string
  getLocale: () => string
  getPlatform: () => Platform | undefined
} | null = null

export function setContextGetters(getters: {
  getTheme: () => string
  getLocale: () => string
  getPlatform: () => Platform | undefined
}) {
  contextGetters = getters
}

export async function handleContextMethodAsync(request: ExtensionRequest) {
  switch (request.method) {
    case HAEXTENSION_METHODS.context.get:
      if (!contextGetters) {
        throw new Error(
          'Context not initialized. Make sure useExtensionMessageHandler is called in a component.',
        )
      }
      return {
        theme: contextGetters.getTheme(),
        locale: contextGetters.getLocale(),
        platform: contextGetters.getPlatform(),
      }

    default:
      throw new Error(`Unknown context method: ${request.method}`)
  }
}

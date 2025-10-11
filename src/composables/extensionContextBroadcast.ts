// composables/extensionContextBroadcast.ts
// NOTE: This composable is deprecated. Use tabsStore.broadcastToAllTabs() instead.
// Keeping for backwards compatibility.

import { getExtensionWindow } from './extensionMessageHandler'

export const useExtensionContextBroadcast = () => {
  // Globaler State für Extension IDs statt IFrames
  const extensionIds = useState<Set<string>>(
    'extension-ids',
    () => new Set(),
  )

  const registerExtensionIframe = (_iframe: HTMLIFrameElement, extensionId: string) => {
    extensionIds.value.add(extensionId)
  }

  const unregisterExtensionIframe = (_iframe: HTMLIFrameElement, extensionId: string) => {
    extensionIds.value.delete(extensionId)
  }

  const broadcastContextChange = (context: {
    theme: string
    locale: string
    platform: string
  }) => {
    const message = {
      type: 'context.changed',
      data: { context },
      timestamp: Date.now(),
    }

    extensionIds.value.forEach((extensionId) => {
      const win = getExtensionWindow(extensionId)
      if (win) {
        win.postMessage(message, '*')
      }
    })
  }

  const broadcastSearchRequest = (query: string, requestId: string) => {
    const message = {
      type: 'search.request',
      data: {
        query: { query, limit: 10 },
        requestId,
      },
      timestamp: Date.now(),
    }

    extensionIds.value.forEach((extensionId) => {
      const win = getExtensionWindow(extensionId)
      if (win) {
        win.postMessage(message, '*')
      }
    })
  }

  return {
    registerExtensionIframe,
    unregisterExtensionIframe,
    broadcastContextChange,
    broadcastSearchRequest,
  }
}

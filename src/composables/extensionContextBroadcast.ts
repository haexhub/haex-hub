// composables/extensionContextBroadcast.ts
export const useExtensionContextBroadcast = () => {
  // Globaler State für alle aktiven IFrames
  const extensionIframes = useState<Set<HTMLIFrameElement>>(
    'extension-iframes',
    () => new Set(),
  )

  const registerExtensionIframe = (iframe: HTMLIFrameElement) => {
    extensionIframes.value.add(iframe)
  }

  const unregisterExtensionIframe = (iframe: HTMLIFrameElement) => {
    extensionIframes.value.delete(iframe)
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

    extensionIframes.value.forEach((iframe) => {
      iframe.contentWindow?.postMessage(message, '*')
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

    extensionIframes.value.forEach((iframe) => {
      iframe.contentWindow?.postMessage(message, '*')
    })
  }

  return {
    registerExtensionIframe,
    unregisterExtensionIframe,
    broadcastContextChange,
    broadcastSearchRequest,
  }
}

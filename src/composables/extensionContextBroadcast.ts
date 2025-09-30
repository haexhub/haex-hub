/**
 * Broadcasts context changes to all active extensions
 */
export const useExtensionContextBroadcast = () => {
  const extensionIframes = ref<HTMLIFrameElement[]>([])

  const registerExtensionIframe = (iframe: HTMLIFrameElement) => {
    extensionIframes.value.push(iframe)
  }

  const unregisterExtensionIframe = (iframe: HTMLIFrameElement) => {
    extensionIframes.value = extensionIframes.value.filter((f) => f !== iframe)
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

export interface ResourceRequestDetails {
  url: string
  resourceType: string
  tabId?: string
  frameId?: number
}

export interface ResourceRequestResult {
  cancel: boolean
  redirectUrl?: string
}

export interface ContentScript {
  code: string
  matches?: string[]
  runAt?: 'document_start' | 'document_end' | 'document_idle'
}

export interface Extension {
  id: string
  name: string
  version: string
  description?: string
  processNavigation?: (url: string) => boolean
  processResourceRequest?: (
    details: ResourceRequestDetails,
  ) => ResourceRequestResult
  contentScripts?: ContentScript[]
}

export const useBrowserExtensionStore = defineStore(
  'useBrowserExtensionStore',
  () => {
    const extensions = ref<Extension[]>([])
    const isInitialized = ref<boolean>(false)

    return {
      extensions,
      isInitialized,
      initializeAsync,
      processNavigation,
      injectContentScripts,
    }
  },
)

const initializeAsync = async () => {
  const { isInitialized } = storeToRefs(useBrowserExtensionStore())
  return
  if (isInitialized.value) return

  // Lade Erweiterungen aus dem Erweiterungsverzeichnis
  try {
    const extensions = await loadExtensionsAsync()
    for (const extension of extensions) {
      registerExtension(extension)
    }

    isInitialized.value = true
    console.log(`${extensions.length} Erweiterungen geladen`)
  } catch (error) {
    console.error('Fehler beim Laden der Erweiterungen:', error)
  }
}

const loadExtensionsAsync = async (): Promise<Extension[]> => {
  // In einer realen Implementierung würden Sie hier Erweiterungen aus einem Verzeichnis laden
  // Für dieses Beispiel verwenden wir hartcodierte Erweiterungen
  /* const adBlocker = (await import('./ad-blocker')).default;
    const trackerBlocker = (await import('./tracker-blocker')).default; */

  return []
}

const registerExtension = (extension: Extension): boolean => {
  const { extensions } = storeToRefs(useBrowserExtensionStore())
  if (!extension.id || !extension.name) {
    console.error('Ungültige Erweiterung:', extension)
    return false
  }

  console.log(`Erweiterung registriert: ${extension.name}`)
  extensions.value.push(extension)
  return true
}

const processNavigation = () => {
  return true
}

const injectContentScripts = () => {}

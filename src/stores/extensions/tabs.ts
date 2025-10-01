// stores/extensions/tabs.ts
import type { IHaexHubExtension } from '~/types/haexhub'

interface ExtensionTab {
  extension: IHaexHubExtension
  iframe: HTMLIFrameElement | null
  isVisible: boolean
  lastAccessed: number
}

export const useExtensionTabsStore = defineStore('extensionTabsStore', () => {
  // State
  const openTabs = ref(new Map<string, ExtensionTab>())
  const activeTabId = ref<string | null>(null)

  // Getters
  const activeTab = computed(() => {
    if (!activeTabId.value) return null
    return openTabs.value.get(activeTabId.value) || null
  })

  const tabCount = computed(() => openTabs.value.size)

  const sortedTabs = computed(() => {
    return Array.from(openTabs.value.values()).sort(
      (a, b) => b.lastAccessed - a.lastAccessed,
    )
  })

  // Actions
  const openTab = (extensionId: string) => {
    // Hole Extension-Info aus dem anderen Store
    const extensionsStore = useExtensionsStore()
    const extension = extensionsStore.availableExtensions.find(
      (ext) => ext.id === extensionId,
    )

    if (!extension) {
      console.error(`Extension ${extensionId} nicht gefunden`)
      return
    }

    // Bereits geöffnet? Nur aktivieren
    if (openTabs.value.has(extensionId)) {
      setActiveTab(extensionId)
      return
    }

    // Limit: Max 10 Tabs
    if (openTabs.value.size >= 10) {
      const oldestInactive = sortedTabs.value
        .filter((tab) => tab.extension.id !== activeTabId.value)
        .pop()

      if (oldestInactive) {
        closeTab(oldestInactive.extension.id)
      }
    }

    // Neuen Tab erstellen
    openTabs.value.set(extensionId, {
      extension,
      iframe: null,
      isVisible: false,
      lastAccessed: Date.now(),
    })

    setActiveTab(extensionId)
  }

  const setActiveTab = (extensionId: string) => {
    // Verstecke aktuellen Tab
    if (activeTabId.value && openTabs.value.has(activeTabId.value)) {
      const currentTab = openTabs.value.get(activeTabId.value)!
      currentTab.isVisible = false
    }

    // Zeige neuen Tab
    const newTab = openTabs.value.get(extensionId)
    if (newTab) {
      newTab.isVisible = true
      newTab.lastAccessed = Date.now()
      activeTabId.value = extensionId
    }
  }

  const closeTab = (extensionId: string) => {
    const tab = openTabs.value.get(extensionId)
    if (!tab) return

    // IFrame entfernen
    tab.iframe?.remove()
    openTabs.value.delete(extensionId)

    // Nächsten Tab aktivieren
    if (activeTabId.value === extensionId) {
      const remaining = sortedTabs.value
      const nextTab = remaining[0]

      if (nextTab) {
        setActiveTab(nextTab.extension.id)
      } else {
        activeTabId.value = null
      }
    }
  }

  const registerIFrame = (extensionId: string, iframe: HTMLIFrameElement) => {
    const tab = openTabs.value.get(extensionId)
    if (tab) {
      tab.iframe = iframe
    }
  }

  const broadcastToAllTabs = (message: unknown) => {
    openTabs.value.forEach(({ iframe }) => {
      iframe?.contentWindow?.postMessage(message, '*')
    })
  }

  const closeAllTabs = () => {
    openTabs.value.forEach((tab) => tab.iframe?.remove())
    openTabs.value.clear()
    activeTabId.value = null
  }

  return {
    // State
    openTabs,
    activeTabId,
    // Getters
    activeTab,
    tabCount,
    sortedTabs,
    // Actions
    openTab,
    setActiveTab,
    closeTab,
    registerIFrame,
    broadcastToAllTabs,
    closeAllTabs,
  }
})

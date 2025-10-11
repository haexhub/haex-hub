// stores/extensions/tabs.ts
import type { IHaexHubExtension } from '~/types/haexhub'
import { getExtensionWindow } from '~/composables/extensionMessageHandler'

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

    // Check if extension is enabled
    if (!extension.enabled) {
      console.warn(`Extension ${extensionId} ist deaktiviert und kann nicht geöffnet werden`)
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
      const now = Date.now()
      const inactiveDuration = now - newTab.lastAccessed
      const TEN_MINUTES = 10 * 60 * 1000

      // Reload iframe if inactive for more than 10 minutes
      if (inactiveDuration > TEN_MINUTES && newTab.iframe) {
        console.log(`[TabStore] Reloading extension ${extensionId} after ${Math.round(inactiveDuration / 1000)}s inactivity`)
        const currentSrc = newTab.iframe.src
        newTab.iframe.src = 'about:blank'
        // Small delay to ensure reload
        setTimeout(() => {
          if (newTab.iframe) {
            newTab.iframe.src = currentSrc
          }
        }, 50)
      }

      newTab.isVisible = true
      newTab.lastAccessed = now
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
    openTabs.value.forEach(({ extension }) => {
      // Use sandbox-compatible window reference
      const win = getExtensionWindow(extension.id)
      if (win) {
        win.postMessage(message, '*')
      }
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

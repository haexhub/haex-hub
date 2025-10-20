import { defineAsyncComponent, type Component } from 'vue'

export interface IWindow {
  id: string
  workspaceId: string // Window belongs to a specific workspace
  type: 'system' | 'extension'
  sourceId: string // extensionId or systemWindowId (depends on type)
  title: string
  icon?: string | null
  x: number
  y: number
  width: number
  height: number
  isMinimized: boolean
  zIndex: number
  // Animation source position (icon position)
  sourceX?: number
  sourceY?: number
  sourceWidth?: number
  sourceHeight?: number
  // Animation state
  isOpening?: boolean
  isClosing?: boolean
}

export interface SystemWindowDefinition {
  id: string
  name: string
  icon: string
  component: Component
  defaultWidth: number
  defaultHeight: number
  resizable?: boolean
  singleton?: boolean // Nur eine Instanz erlaubt?
}

export const useWindowManagerStore = defineStore('windowManager', () => {
  const windows = ref<IWindow[]>([])
  const activeWindowId = ref<string | null>(null)
  const nextZIndex = ref(100)

  // System Windows Registry
  const systemWindows: Record<string, SystemWindowDefinition> = {
    settings: {
      id: 'settings',
      name: 'Settings',
      icon: 'i-mdi-cog',
      component: defineAsyncComponent(
        () => import('@/components/haex/system/settings.vue'),
      ) as Component,
      defaultWidth: 800,
      defaultHeight: 600,
      resizable: true,
      singleton: true,
    },
    marketplace: {
      id: 'marketplace',
      name: 'Marketplace',
      icon: 'i-mdi-store',
      component: defineAsyncComponent(
        () => import('@/components/haex/system/marketplace.vue'),
      ),
      defaultWidth: 1000,
      defaultHeight: 700,
      resizable: true,
      singleton: false,
    },
  }

  const getSystemWindow = (id: string): SystemWindowDefinition | undefined => {
    return systemWindows[id]
  }

  const getAllSystemWindows = (): SystemWindowDefinition[] => {
    return Object.values(systemWindows)
  }

  // Window animation settings
  const windowAnimationDuration = ref(600) // in milliseconds (matches Tailwind duration-600)

  // Get windows for current workspace only
  const currentWorkspaceWindows = computed(() => {
    if (!useWorkspaceStore().currentWorkspace) return []
    return windows.value.filter(
      (w) => w.workspaceId === useWorkspaceStore().currentWorkspace?.id,
    )
  })

  const windowsByWorkspaceId = (workspaceId: string) =>
    computed(() =>
      windows.value.filter((window) => window.workspaceId === workspaceId),
    )

  const moveWindowsToWorkspace = (
    fromWorkspaceId: string,
    toWorkspaceId: string,
  ) => {
    const windowsFrom = windowsByWorkspaceId(fromWorkspaceId)
    windowsFrom.value.forEach((window) => (window.workspaceId = toWorkspaceId))
  }

  const openWindowAsync = async ({
    height = 800,
    icon = '',
    sourceId,
    sourcePosition,
    title,
    type,
    width = 600,
    workspaceId,
  }: {
    height?: number
    icon?: string | null
    sourceId: string
    sourcePosition?: { x: number; y: number; width: number; height: number }
    title?: string
    type: 'system' | 'extension'
    width?: number
    workspaceId?: string
  }) => {
    try {
      // Wenn kein workspaceId angegeben ist, nutze die current workspace
      const targetWorkspaceId =
        workspaceId || useWorkspaceStore().currentWorkspace?.id

      if (!targetWorkspaceId) {
        console.error('Cannot open window: No active workspace')
        return
      }

      const workspace = useWorkspaceStore().workspaces?.find(
        (w) => w.id === targetWorkspaceId,
      )
      if (!workspace) {
        console.error('Cannot open window: Invalid workspace')
        return
      }

      // System Window specific handling
      if (type === 'system') {
        const systemWindowDef = getSystemWindow(sourceId)
        if (!systemWindowDef) {
          console.error(`System window '${sourceId}' not found in registry`)
          return
        }

        // Singleton check: If already open, activate existing window
        if (systemWindowDef.singleton) {
          const existingWindow = windows.value.find(
            (w) => w.type === 'system' && w.sourceId === sourceId,
          )
          if (existingWindow) {
            activateWindow(existingWindow.id)
            return existingWindow.id
          }
        }

        // Use system window defaults
        title = title ?? systemWindowDef.name
        icon = icon ?? systemWindowDef.icon
        width = width ?? systemWindowDef.defaultWidth
        height = height ?? systemWindowDef.defaultHeight
      }

      // Create new window
      const windowId = crypto.randomUUID()

      // Calculate viewport-aware size
      const viewportWidth = window.innerWidth
      const viewportHeight = window.innerHeight

      const windowWidth = width > viewportWidth ? viewportWidth : width
      const windowHeight = height > viewportHeight ? viewportHeight : height

      // Calculate centered position with cascading offset (only count windows in current workspace)
      const offset = currentWorkspaceWindows.value.length * 30
      const centerX = Math.max(0, (viewportWidth - windowWidth) / 1 / 3)
      const centerY = Math.max(0, (viewportHeight - windowHeight) / 1 / 3)
      const x = Math.min(centerX + offset, viewportWidth - windowWidth)
      const y = Math.min(centerY + offset, viewportHeight - windowHeight)

      const newWindow: IWindow = {
        id: windowId,
        workspaceId: workspace.id,
        type,
        sourceId,
        title: title!,
        icon,
        x,
        y,
        width: windowWidth,
        height: windowHeight,
        isMinimized: false,
        zIndex: nextZIndex.value++,
        sourceX: sourcePosition?.x,
        sourceY: sourcePosition?.y,
        sourceWidth: sourcePosition?.width,
        sourceHeight: sourcePosition?.height,
        isOpening: true,
        isClosing: false,
      }

      windows.value.push(newWindow)
      activeWindowId.value = windowId

      // Remove opening flag after animation
      setTimeout(() => {
        const window = windows.value.find((w) => w.id === windowId)
        if (window) {
          window.isOpening = false
        }
      }, windowAnimationDuration.value)

      return windowId
    } catch (error) {
      console.error('Error opening window:', error)
      // Optional: Fehler weiterwerfen wenn nötig
      throw error
    }
  }

  /*****************************************************************************************************
   * TODO: Momentan werden die Fenster einfach nur geschlossen.
   * In Zukunft sollte aber vorher ein close event an die Erweiterungen via postMessage geschickt werden,
   * so dass die Erweiterungen darauf reagieren können, um eventuell ungespeicherte Daten zu sichern
   *****************************************************************************************************/
  const closeWindow = (windowId: string) => {
    const window = windows.value.find((w) => w.id === windowId)
    if (!window) return

    // Start closing animation
    window.isClosing = true

    // Remove window after animation completes
    setTimeout(() => {
      const index = windows.value.findIndex((w) => w.id === windowId)
      if (index !== -1) {
        windows.value.splice(index, 1)

        // If closed window was active, activate the topmost window
        if (activeWindowId.value === windowId) {
          if (windows.value.length > 0) {
            const topWindow = windows.value.reduce((max, w) =>
              w.zIndex > max.zIndex ? w : max,
            )
            activeWindowId.value = topWindow.id
          } else {
            activeWindowId.value = null
          }
        }
      }
    }, windowAnimationDuration.value)
  }

  const minimizeWindow = (windowId: string) => {
    const window = windows.value.find((w) => w.id === windowId)
    if (window) {
      window.isMinimized = true
    }
  }

  const restoreWindow = (windowId: string) => {
    const window = windows.value.find((w) => w.id === windowId)
    if (window) {
      window.isMinimized = false
      activateWindow(windowId)
    }
  }

  const activateWindow = (windowId: string) => {
    const window = windows.value.find((w) => w.id === windowId)
    if (window) {
      window.zIndex = nextZIndex.value++
      activeWindowId.value = windowId
    }
  }

  const updateWindowPosition = (windowId: string, x: number, y: number) => {
    const window = windows.value.find((w) => w.id === windowId)
    if (window) {
      window.x = x
      window.y = y
    }
  }

  const updateWindowSize = (
    windowId: string,
    width: number,
    height: number,
  ) => {
    const window = windows.value.find((w) => w.id === windowId)
    if (window) {
      window.width = width
      window.height = height
    }
  }

  const isWindowActive = (windowId: string) => {
    return activeWindowId.value === windowId
  }

  const getVisibleWindows = computed(() => {
    return currentWorkspaceWindows.value.filter((w) => !w.isMinimized)
  })

  const getMinimizedWindows = computed(() => {
    return currentWorkspaceWindows.value.filter((w) => w.isMinimized)
  })

  return {
    activateWindow,
    activeWindowId,
    closeWindow,
    currentWorkspaceWindows,
    getAllSystemWindows,
    getMinimizedWindows,
    getSystemWindow,
    getVisibleWindows,
    isWindowActive,
    minimizeWindow,
    moveWindowsToWorkspace,
    openWindowAsync,
    restoreWindow,
    updateWindowPosition,
    updateWindowSize,
    windowAnimationDuration,
    windows,
    windowsByWorkspaceId,
  }
})

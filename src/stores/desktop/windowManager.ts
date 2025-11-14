import { defineAsyncComponent, type Component } from 'vue'
import { getFullscreenDimensions } from '~/utils/viewport'
import { isDesktop } from '~/utils/platform'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { EXTENSION_WINDOW_CLOSED } from '~/constants/events'

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

  // Window Overview State
  const showWindowOverview = ref(false)

  // Computed: Count of all open windows (including minimized)
  const openWindowsCount = computed(() => windows.value.length)

  // Window Dragging State (for drag & drop to workspaces)
  const draggingWindowId = ref<string | null>(null)

  // System Windows Registry
  const systemWindows: Record<string, SystemWindowDefinition> = {
    developer: {
      id: 'developer',
      name: 'Developer',
      icon: 'i-hugeicons-developer',
      component: defineAsyncComponent(
        () => import('@/components/haex/system/developer.vue'),
      ) as Component,
      defaultWidth: 800,
      defaultHeight: 600,
      resizable: true,
      singleton: true,
    },
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
    debugLogs: {
      id: 'debugLogs',
      name: 'Debug Logs',
      icon: 'i-heroicons-bug-ant',
      component: defineAsyncComponent(
        () => import('@/components/haex/system/debug-logs.vue'),
      ),
      defaultWidth: 1000,
      defaultHeight: 700,
      resizable: true,
      singleton: true,
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
      // Desktop: Check extension's display_mode preference
      if (type === 'extension') {
        const extensionsStore = useExtensionsStore()
        const extension = extensionsStore.availableExtensions.find(
          (e) => e.id === sourceId,
        )
        const finalTitle = title ?? extension?.name ?? 'Extension'

        // Determine if we should use native window based on display_mode and platform
        const displayMode = extension?.displayMode ?? 'auto'
        const shouldUseNativeWindow =
          (displayMode === 'window') ||
          (displayMode === 'auto' && isDesktop())

        console.log('[windowManager] Extension display mode check:', {
          extensionId: sourceId,
          extensionName: extension?.name,
          displayMode,
          isDesktop: isDesktop(),
          shouldUseNativeWindow,
        })

        // Desktop: Extensions can run in native WebviewWindows (separate processes)
        if (isDesktop() && shouldUseNativeWindow) {
          try {
            console.log('[windowManager] Opening native window with sourceId:', sourceId)
            console.log('[windowManager] Extension object:', extension)
            // Backend generates and returns the window_id
            const windowId = await invoke<string>('open_extension_webview_window', {
              extensionId: sourceId,
              title: finalTitle,
              width,
              height,
              x: undefined, // Let OS handle positioning
              y: undefined,
            })

            // Store minimal metadata for tracking (no UI management needed on desktop)
            const newWindow: IWindow = {
              id: windowId, // Use window_id from backend as ID
              workspaceId: '', // Not used on desktop
              type,
              sourceId,
              title: finalTitle,
              icon,
              x: 0,
              y: 0,
              width,
              height,
              isMinimized: false,
              zIndex: 0,
              isOpening: false,
              isClosing: false,
            }
            windows.value.push(newWindow)

            return windowId
          } catch (error) {
            console.error('Failed to open native extension window:', error)
            throw error
          }
        }

        // If display_mode is 'iframe' or we're not on desktop, fall through to iframe logic
      }

      // Mobile: Full UI-based window management (original logic)
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
      const viewportHeight = window.innerHeight - 60

      console.log('viewportHeight', window.innerHeight, viewportHeight)

      // Check if we're on a small screen
      const { isSmallScreen } = useUiStore()

      let windowWidth: number
      let windowHeight: number
      let x: number
      let y: number

      if (isSmallScreen) {
        // On small screens, make window fullscreen starting at 0,0
        // Use helper function to calculate correct dimensions with safe areas
        const fullscreen = getFullscreenDimensions()
        x = fullscreen.x
        y = fullscreen.y
        windowWidth = fullscreen.width
        windowHeight = fullscreen.height
      } else {
        // On larger screens, use normal sizing and positioning
        windowHeight = Math.min(height, viewportHeight)

        // Adjust width proportionally if needed (optional)
        const aspectRatio = width / height
        windowWidth = Math.min(
          width,
          viewportWidth,
          windowHeight * aspectRatio,
        )

        // Calculate centered position with cascading offset (only count windows in current workspace)
        const offset = currentWorkspaceWindows.value.length * 30
        const centerX = Math.max(0, (viewportWidth - windowWidth) / 1 / 3)
        const centerY = Math.max(0, (viewportHeight - windowHeight) / 1 / 3)
        x = Math.min(centerX + offset, viewportWidth - windowWidth)
        y = Math.min(centerY + offset, viewportHeight - windowHeight)
      }

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
  const closeWindow = async (windowId: string) => {
    const window = windows.value.find((w) => w.id === windowId)
    if (!window) return

    // Desktop: Close native WebviewWindow for extensions (only if it's actually a native window)
    // Check if extension is using native window mode (not iframe)
    if (isDesktop() && window.type === 'extension') {
      const extensionsStore = useExtensionsStore()
      const extension = extensionsStore.availableExtensions.find(
        (e) => e.id === window.sourceId,
      )
      const displayMode = extension?.displayMode ?? 'auto'
      const isNativeWindow =
        (displayMode === 'window') ||
        (displayMode === 'auto' && isDesktop())

      // Only try to close native window if it's actually running as native window
      if (isNativeWindow) {
        try {
          await invoke('close_extension_webview_window', { windowId })
          // Backend will emit event, our listener will update frontend tracking
        } catch (error) {
          console.error('Failed to close native extension window:', error)
        }
        return
      }
      // If not a native window, fall through to iframe cleanup below
    }

    // Mobile: Animated close with iframe cleanup
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
      window.isMinimized = false
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

  // Desktop: Listen for native window close events from Tauri
  // Backend is source of truth, frontend is read-only mirror for tracking
  let _unlistenWindowClosed: UnlistenFn | null = null

  const setupDesktopEventListenersAsync = async () => {
    if (!isDesktop()) return

    // Listen for native WebviewWindow close events from backend
    _unlistenWindowClosed = await listen<string>(
      EXTENSION_WINDOW_CLOSED,
      (event) => {
        const windowId = event.payload
        console.log(`Native extension window closed: ${windowId}`)

        // Remove from frontend tracking (read-only mirror of backend state)
        const index = windows.value.findIndex((w) => w.id === windowId)
        if (index !== -1) {
          windows.value.splice(index, 1)
        }
      },
    )
  }

  // Setup listeners on store creation (only on desktop)
  if (isDesktop()) {
    setupDesktopEventListenersAsync()
  }

  return {
    activateWindow,
    activeWindowId,
    closeWindow,
    currentWorkspaceWindows,
    draggingWindowId,
    getAllSystemWindows,
    getMinimizedWindows,
    getSystemWindow,
    getVisibleWindows,
    isWindowActive,
    minimizeWindow,
    moveWindowsToWorkspace,
    openWindowAsync,
    openWindowsCount,
    restoreWindow,
    showWindowOverview,
    updateWindowPosition,
    updateWindowSize,
    windowAnimationDuration,
    windows,
    windowsByWorkspaceId,
  }
})

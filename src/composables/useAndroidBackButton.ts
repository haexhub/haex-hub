import { onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { platform } from '@tauri-apps/plugin-os'
import { getCurrentWindow } from '@tauri-apps/api/window'

/**
 * Handles Android back button to navigate within the app instead of closing it
 * Mimics browser behavior: navigate back if possible, close app if on first page
 */
export function useAndroidBackButton() {
  const router = useRouter()
  const historyStack = ref<string[]>([])
  let unlisten: (() => void) | null = null

  // Track navigation history manually
  router.afterEach((to, from) => {
    console.log('[AndroidBack] Navigation:', { to: to.path, from: from.path, stackSize: historyStack.value.length })

    // If navigating forward (new page)
    if (from.path && to.path !== from.path && !historyStack.value.includes(to.path)) {
      historyStack.value.push(from.path)
      console.log('[AndroidBack] Added to stack:', from.path, 'Stack:', historyStack.value)
    }
  })

  onMounted(async () => {
    const os = platform()

    if (os === 'android') {
      const appWindow = getCurrentWindow()

      // Listen to close requested event (triggered by Android back button)
      unlisten = await appWindow.onCloseRequested(async (event) => {
        console.log('[AndroidBack] Back button pressed, stack size:', historyStack.value.length)

        // Check if we have history
        if (historyStack.value.length > 0) {
          // Prevent window from closing
          event.preventDefault()

          // Remove current page from stack
          historyStack.value.pop()
          console.log('[AndroidBack] Going back, new stack size:', historyStack.value.length)

          // Navigate back in router
          router.back()
        } else {
          console.log('[AndroidBack] No history, allowing app to close')
        }
        // If no history, allow default behavior (app closes)
      })
    }
  })

  onUnmounted(() => {
    if (unlisten) {
      unlisten()
    }
  })
}

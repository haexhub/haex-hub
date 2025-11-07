<template>
  <UiDrawer
    v-model:open="localShowWindowOverview"
    direction="bottom"
    :title="t('modal.title')"
    :description="t('modal.description')"
  >
    <template #content>
      <div class="h-full overflow-y-auto p-6 justify-center flex">
        <!-- Window Thumbnails Flex Layout -->

        <div
          v-if="windows.length > 0"
          class="flex flex-wrap gap-6 justify-center-safe items-start"
        >
          <div
            v-for="window in windows"
            :key="window.id"
            class="relative group cursor-pointer"
          >
            <!-- Window Title Bar -->
            <div class="flex items-center gap-3 mb-2 px-2">
              <UIcon
                v-if="window.icon"
                :name="window.icon"
                class="size-5 shrink-0"
              />
              <div class="flex-1 min-w-0">
                <p class="font-semibold text-sm truncate">
                  {{ window.title }}
                </p>
              </div>
              <!-- Minimized Badge -->
              <UBadge
                v-if="window.isMinimized"
                color="info"
                size="xs"
                :title="t('minimized')"
              />
            </div>

            <!-- Scaled Window Preview Container / Teleport Target -->
            <div
              :id="`window-preview-${window.id}`"
              class="relative bg-gray-100 dark:bg-gray-900 rounded-xl overflow-hidden border-2 border-gray-200 dark:border-gray-700 group-hover:border-primary-500 transition-all shadow-lg"
              :style="getCardStyle(window)"
              @click="handleRestoreAndActivateWindow(window.id)"
            >
              <!-- Hover Overlay -->
              <div
                class="absolute inset-0 bg-primary-500/10 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-40"
              />
            </div>
          </div>
        </div>

        <!-- Empty State -->
        <div
          v-else
          class="flex flex-col items-center justify-center py-12 text-gray-500 dark:text-gray-400"
        >
          <UIcon
            name="i-heroicons-window"
            class="size-16 mb-4 shrink-0"
          />
          <p class="text-lg font-medium">No windows open</p>
          <p class="text-sm">
            Open an extension or system window to see it here
          </p>
        </div>
      </div>
    </template>
  </UiDrawer>
</template>

<script setup lang="ts">
const { t } = useI18n()

const windowManager = useWindowManagerStore()
const workspaceStore = useWorkspaceStore()

const { showWindowOverview, windows } = storeToRefs(windowManager)

// Local computed for two-way binding with UModal
const localShowWindowOverview = computed({
  get: () => showWindowOverview.value,
  set: (value) => {
    showWindowOverview.value = value
  },
})

const handleRestoreAndActivateWindow = (windowId: string) => {
  const window = windowManager.windows.find((w) => w.id === windowId)
  if (!window) return

  // Switch to the workspace where this window is located
  if (window.workspaceId) {
    workspaceStore.slideToWorkspace(window.workspaceId)
  }

  // If window is minimized, restore it first
  if (window.isMinimized) {
    windowManager.restoreWindow(windowId)
  } else {
    // If not minimized, just activate it
    windowManager.activateWindow(windowId)
  }

  // Close the overview
  localShowWindowOverview.value = false
}

// Store original window sizes and positions to restore after overview closes
const originalWindowState = ref<
  Map<string, { width: number; height: number; x: number; y: number }>
>(new Map())

// Min/Max dimensions for preview cards
const MIN_PREVIEW_WIDTH = 300
const MAX_PREVIEW_WIDTH = 600
const MIN_PREVIEW_HEIGHT = 225
const MAX_PREVIEW_HEIGHT = 450

// Calculate card size and scale based on window dimensions
const getCardStyle = (window: (typeof windows.value)[0]) => {
  const scaleX = MAX_PREVIEW_WIDTH / window.width
  const scaleY = MAX_PREVIEW_HEIGHT / window.height
  const scale = Math.min(scaleX, scaleY, 1) // Never scale up, only down

  // Calculate scaled dimensions
  const scaledWidth = window.width * scale
  const scaledHeight = window.height * scale

  // Ensure minimum card size
  let finalScale = scale
  if (scaledWidth < MIN_PREVIEW_WIDTH) {
    finalScale = MIN_PREVIEW_WIDTH / window.width
  }
  if (scaledHeight < MIN_PREVIEW_HEIGHT) {
    finalScale = Math.max(finalScale, MIN_PREVIEW_HEIGHT / window.height)
  }

  const cardWidth = window.width * finalScale
  const cardHeight = window.height * finalScale

  return {
    width: `${cardWidth}px`,
    height: `${cardHeight}px`,
    '--window-scale': finalScale, // CSS variable for scale
  }
}

// Watch for overview closing to restore windows
watch(localShowWindowOverview, async (isOpen, wasOpen) => {
  if (!isOpen && wasOpen) {
    console.log('[WindowOverview] Overview closed, restoring windows...')

    // Restore original window state
    for (const window of windows.value) {
      const originalState = originalWindowState.value.get(window.id)
      if (originalState) {
        console.log(
          `[WindowOverview] Restoring window ${window.id} to:`,
          originalState,
        )

        windowManager.updateWindowSize(
          window.id,
          originalState.width,
          originalState.height,
        )
        windowManager.updateWindowPosition(
          window.id,
          originalState.x,
          originalState.y,
        )
      }
    }
    originalWindowState.value.clear()
  }
})

// Watch for overview opening to store original state
watch(
  () => localShowWindowOverview.value && windows.value.length,
  (shouldStore) => {
    if (shouldStore && originalWindowState.value.size === 0) {
      console.log('[WindowOverview] Storing original window states...')

      for (const window of windows.value) {
        console.log(`[WindowOverview] Window ${window.id}:`, {
          originalSize: { width: window.width, height: window.height },
          originalPos: { x: window.x, y: window.y },
        })

        originalWindowState.value.set(window.id, {
          width: window.width,
          height: window.height,
          x: window.x,
          y: window.y,
        })
      }
    }
  },
)
</script>

<i18n lang="yaml">
de:
  modal:
    title: Fensterübersicht
    description: Übersicht aller offenen Fenster auf allen Workspaces

  minimized: Minimiert

en:
  modal:
    title: Window Overview
    description: Overview of all open windows on all workspaces

  minimized: Minimized
</i18n>

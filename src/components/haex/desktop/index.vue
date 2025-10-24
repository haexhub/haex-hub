<template>
  <div
    ref="desktopEl"
    class="w-full h-full relative overflow-hidden isolate"
  >
    <Swiper
      :modules="[SwiperNavigation]"
      :slides-per-view="1"
      :space-between="0"
      :initial-slide="currentWorkspaceIndex"
      :speed="300"
      :touch-angle="45"
      :no-swiping="true"
      no-swiping-class="no-swipe"
      :allow-touch-move="allowSwipe"
      class="w-full h-full"
      @swiper="onSwiperInit"
      @slide-change="onSlideChange"
      direction="vertical"
    >
      <SwiperSlide
        v-for="workspace in workspaces"
        :key="workspace.id"
        class="w-full h-full"
      >
        <div
          class="w-full h-full relative isolate"
          @click.self.stop="handleDesktopClick"
          @mousedown.left.self="handleAreaSelectStart"
        >
          <!-- Grid Pattern Background -->
          <div
            class="absolute inset-0 pointer-events-none opacity-30"
            :style="{
              backgroundImage:
                'linear-gradient(rgba(0, 0, 0, 0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(0, 0, 0, 0.1) 1px, transparent 1px)',
              backgroundSize: '32px 32px',
            }"
          />

          <!-- Snap Dropzones (only visible when window drag near edge) -->
          <Transition name="fade">
            <div
              v-if="showLeftSnapZone"
              class="absolute left-0 top-0 bottom-0 w-1/2 bg-blue-500/20 border-2 border-blue-500 pointer-events-none backdrop-blur-sm z-40"
            />
          </Transition>
          <Transition name="fade">
            <div
              v-if="showRightSnapZone"
              class="absolute right-0 top-0 bottom-0 w-1/2 bg-blue-500/20 border-2 border-blue-500 pointer-events-none backdrop-blur-sm z-40"
            />
          </Transition>

          <!-- Area Selection Box -->
          <div
            v-if="isAreaSelecting"
            class="absolute bg-blue-500/20 border-2 border-blue-500 pointer-events-none z-30"
            :style="selectionBoxStyle"
          />

          <!-- Icons for this workspace -->
          <HaexDesktopIcon
            v-for="item in getWorkspaceIcons(workspace.id)"
            :id="item.id"
            :key="item.id"
            :item-type="item.itemType"
            :reference-id="item.referenceId"
            :initial-x="item.positionX"
            :initial-y="item.positionY"
            :label="item.label"
            :icon="item.icon"
            class="no-swipe"
            @position-changed="handlePositionChanged"
            @drag-start="handleDragStart"
            @drag-end="handleDragEnd"
          />

          <!-- Windows for this workspace -->
          <template
            v-for="window in getWorkspaceWindows(workspace.id)"
            :key="window.id"
          >
            <!-- Desktop container for when overview is closed -->
            <div
              :id="`desktop-container-${window.id}`"
              class="absolute"
            />

            <!-- Window with dynamic teleport -->
            <Teleport
              :to="
                windowManager.showWindowOverview &&
                overviewWindowState.has(window.id)
                  ? `#window-preview-${window.id}`
                  : `#desktop-container-${window.id}`
              "
            >
              <template
                v-if="
                  windowManager.showWindowOverview &&
                  overviewWindowState.has(window.id)
                "
              >
                <div
                  class="absolute origin-top-left"
                  :style="{
                    transform: `scale(${overviewWindowState.get(window.id)!.scale})`,
                    width: `${overviewWindowState.get(window.id)!.width}px`,
                    height: `${overviewWindowState.get(window.id)!.height}px`,
                  }"
                >
                  <HaexWindow
                    v-show="
                      windowManager.showWindowOverview || !window.isMinimized
                    "
                    :id="window.id"
                    :title="window.title"
                    :icon="window.icon"
                    v-model:x="overviewWindowState.get(window.id)!.x"
                    v-model:y="overviewWindowState.get(window.id)!.y"
                    v-model:width="overviewWindowState.get(window.id)!.width"
                    v-model:height="overviewWindowState.get(window.id)!.height"
                    :is-active="windowManager.isWindowActive(window.id)"
                    :source-x="window.sourceX"
                    :source-y="window.sourceY"
                    :source-width="window.sourceWidth"
                    :source-height="window.sourceHeight"
                    :is-opening="window.isOpening"
                    :is-closing="window.isClosing"
                    class="no-swipe"
                    @close="windowManager.closeWindow(window.id)"
                    @minimize="windowManager.minimizeWindow(window.id)"
                    @activate="windowManager.activateWindow(window.id)"
                    @position-changed="
                      (x, y) =>
                        windowManager.updateWindowPosition(window.id, x, y)
                    "
                    @size-changed="
                      (width, height) =>
                        windowManager.updateWindowSize(window.id, width, height)
                    "
                    @drag-start="handleWindowDragStart(window.id)"
                    @drag-end="handleWindowDragEnd"
                  >
                    <!-- System Window: Render Vue Component -->
                    <component
                      :is="getSystemWindowComponent(window.sourceId)"
                      v-if="window.type === 'system'"
                    />

                    <!-- Extension Window: Render iFrame -->
                    <HaexDesktopExtensionFrame
                      v-else
                      :extension-id="window.sourceId"
                      :window-id="window.id"
                    />
                  </HaexWindow>
                </div>
              </template>
              <HaexWindow
                v-else
                v-show="windowManager.showWindowOverview || !window.isMinimized"
                :id="window.id"
                :title="window.title"
                :icon="window.icon"
                v-model:x="window.x"
                v-model:y="window.y"
                v-model:width="window.width"
                v-model:height="window.height"
                :is-active="windowManager.isWindowActive(window.id)"
                :source-x="window.sourceX"
                :source-y="window.sourceY"
                :source-width="window.sourceWidth"
                :source-height="window.sourceHeight"
                :is-opening="window.isOpening"
                :is-closing="window.isClosing"
                class="no-swipe"
                @close="windowManager.closeWindow(window.id)"
                @minimize="windowManager.minimizeWindow(window.id)"
                @activate="windowManager.activateWindow(window.id)"
                @position-changed="
                  (x, y) => windowManager.updateWindowPosition(window.id, x, y)
                "
                @size-changed="
                  (width, height) =>
                    windowManager.updateWindowSize(window.id, width, height)
                "
                @drag-start="handleWindowDragStart(window.id)"
                @drag-end="handleWindowDragEnd"
              >
                <!-- System Window: Render Vue Component -->
                <component
                  :is="getSystemWindowComponent(window.sourceId)"
                  v-if="window.type === 'system'"
                />

                <!-- Extension Window: Render iFrame -->
                <HaexDesktopExtensionFrame
                  v-else
                  :extension-id="window.sourceId"
                  :window-id="window.id"
                />
              </HaexWindow>
            </Teleport>
          </template>
        </div>
      </SwiperSlide>
    </Swiper>

    <!-- Window Overview Modal -->
    <HaexWindowOverview />

    <!-- Workspace Drawer -->
    <UDrawer
      v-model:open="isOverviewMode"
      direction="left"
      :dismissible="false"
      :overlay="false"
      :modal="false"
      title="Workspaces"
      description="Workspaces"
    >
      <template #content>
        <div class="p-6 h-full overflow-y-auto">
          <UButton
            block
            trailing-icon="mdi-close"
            class="text-2xl font-bold ext-gray-900 dark:text-white mb-4"
            @click="isOverviewMode = false"
          >
            Workspaces
          </UButton>

          <!-- Workspace Cards -->
          <div class="flex flex-col gap-3">
            <HaexWorkspaceCard
              v-for="workspace in workspaces"
              :key="workspace.id"
              :workspace
            />
          </div>

          <!-- Add New Workspace Button -->
          <UButton
            block
            variant="outline"
            class="mt-6"
            @click="handleAddWorkspace"
          >
            <template #leading>
              <UIcon name="i-heroicons-plus" />
            </template>
            New Workspace
          </UButton>
        </div>
      </template>
    </UDrawer>
  </div>
</template>

<script setup lang="ts">
import { Swiper, SwiperSlide } from 'swiper/vue'
import { Navigation } from 'swiper/modules'
import type { Swiper as SwiperType } from 'swiper'
import 'swiper/css'
import 'swiper/css/navigation'

const SwiperNavigation = Navigation

const desktopStore = useDesktopStore()
const extensionsStore = useExtensionsStore()
const windowManager = useWindowManagerStore()
const workspaceStore = useWorkspaceStore()
const { desktopItems } = storeToRefs(desktopStore)
const { availableExtensions } = storeToRefs(extensionsStore)
const {
  currentWorkspace,
  currentWorkspaceIndex,
  workspaces,
  swiperInstance,
  allowSwipe,
  isOverviewMode,
} = storeToRefs(workspaceStore)

const { x: mouseX } = useMouse()

const desktopEl = useTemplateRef('desktopEl')

// Track desktop viewport size reactively
const { width: viewportWidth, height: viewportHeight } =
  useElementSize(desktopEl)

// Provide viewport size to child windows
provide('viewportSize', {
  width: viewportWidth,
  height: viewportHeight,
})

// Area selection state
const isAreaSelecting = ref(false)
const selectionStart = ref({ x: 0, y: 0 })
const selectionEnd = ref({ x: 0, y: 0 })

const selectionBoxStyle = computed(() => {
  const x1 = Math.min(selectionStart.value.x, selectionEnd.value.x)
  const y1 = Math.min(selectionStart.value.y, selectionEnd.value.y)
  const x2 = Math.max(selectionStart.value.x, selectionEnd.value.x)
  const y2 = Math.max(selectionStart.value.y, selectionEnd.value.y)

  return {
    left: `${x1}px`,
    top: `${y1}px`,
    width: `${x2 - x1}px`,
    height: `${y2 - y1}px`,
  }
})

// Drag state for desktop icons
const isDragging = ref(false)
const currentDraggedItemId = ref<string>()
const currentDraggedItemType = ref<string>()
const currentDraggedReferenceId = ref<string>()

// Window drag state for snap zones
const isWindowDragging = ref(false)
const snapEdgeThreshold = 50 // pixels from edge to show snap zone

// Computed visibility for snap zones (uses mouseX from above)
const showLeftSnapZone = computed(() => {
  return isWindowDragging.value && mouseX.value <= snapEdgeThreshold
})

const showRightSnapZone = computed(() => {
  if (!isWindowDragging.value) return false
  const viewportWidth = window.innerWidth
  return mouseX.value >= viewportWidth - snapEdgeThreshold
})

// Get icons for a specific workspace
const getWorkspaceIcons = (workspaceId: string) => {
  return desktopItems.value
    .filter((item) => item.workspaceId === workspaceId)
    .map((item) => {
      if (item.itemType === 'extension') {
        const extension = availableExtensions.value.find(
          (ext) => ext.id === item.referenceId,
        )

        return {
          ...item,
          label: extension?.name || 'Unknown',
          icon: extension?.icon || '',
        }
      }

      if (item.itemType === 'file') {
        // Für später: file handling
        return {
          ...item,
          label: item.referenceId,
          icon: undefined,
        }
      }

      if (item.itemType === 'folder') {
        // Für später: folder handling
        return {
          ...item,
          label: item.referenceId,
          icon: undefined,
        }
      }

      return {
        ...item,
        label: item.referenceId,
        icon: undefined,
      }
    })
}

// Get windows for a specific workspace (including minimized for teleport)
const getWorkspaceWindows = (workspaceId: string) => {
  return windowManager.windows.filter((w) => w.workspaceId === workspaceId)
}

// Get Vue Component for system window
const getSystemWindowComponent = (sourceId: string) => {
  const systemWindow = windowManager.getSystemWindow(sourceId)
  return systemWindow?.component
}

const handlePositionChanged = async (id: string, x: number, y: number) => {
  try {
    await desktopStore.updateDesktopItemPositionAsync(id, x, y)
  } catch (error) {
    console.error('Fehler beim Speichern der Position:', error)
  }
}

const handleDragStart = (id: string, itemType: string, referenceId: string) => {
  isDragging.value = true
  currentDraggedItemId.value = id
  currentDraggedItemType.value = itemType
  currentDraggedReferenceId.value = referenceId
  allowSwipe.value = false // Disable Swiper during icon drag
}

const handleDragEnd = async () => {
  // Cleanup drag state
  isDragging.value = false
  currentDraggedItemId.value = undefined
  currentDraggedItemType.value = undefined
  currentDraggedReferenceId.value = undefined
  allowSwipe.value = true // Re-enable Swiper after drag
}

const handleDesktopClick = () => {
  // Only clear selection if it was a simple click, not an area selection
  // Check if we just finished an area selection (box size > threshold)
  const boxWidth = Math.abs(selectionEnd.value.x - selectionStart.value.x)
  const boxHeight = Math.abs(selectionEnd.value.y - selectionStart.value.y)

  // If box is larger than 5px in any direction, it was an area select, not a click
  if (boxWidth > 5 || boxHeight > 5) {
    return
  }

  desktopStore.clearSelection()
  isOverviewMode.value = false
}

const handleWindowDragStart = (windowId: string) => {
  console.log('[Desktop] handleWindowDragStart:', windowId)
  isWindowDragging.value = true
  windowManager.draggingWindowId = windowId // Set in store for workspace cards
  console.log(
    '[Desktop] draggingWindowId set to:',
    windowManager.draggingWindowId,
  )
  allowSwipe.value = false // Disable Swiper during window drag
}

const handleWindowDragEnd = async () => {
  console.log('[Desktop] handleWindowDragEnd')
  isWindowDragging.value = false
  windowManager.draggingWindowId = null // Clear from store
  allowSwipe.value = true // Re-enable Swiper after drag
}

// Area selection handlers
const handleAreaSelectStart = (e: MouseEvent) => {
  if (!desktopEl.value) return

  const rect = desktopEl.value.getBoundingClientRect()
  const x = e.clientX - rect.left
  const y = e.clientY - rect.top

  isAreaSelecting.value = true
  selectionStart.value = { x, y }
  selectionEnd.value = { x, y }

  // Clear current selection
  desktopStore.clearSelection()
}

// Track mouse movement for area selection
useEventListener(window, 'mousemove', (e: MouseEvent) => {
  if (isAreaSelecting.value && desktopEl.value) {
    const rect = desktopEl.value.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top

    selectionEnd.value = { x, y }

    // Find all items within selection box
    selectItemsInBox()
  }
})

// End area selection
useEventListener(window, 'mouseup', () => {
  if (isAreaSelecting.value) {
    isAreaSelecting.value = false

    // Reset selection coordinates after a short delay
    // This allows handleDesktopClick to still check the box size
    setTimeout(() => {
      selectionStart.value = { x: 0, y: 0 }
      selectionEnd.value = { x: 0, y: 0 }
    }, 100)
  }
})

const selectItemsInBox = () => {
  const x1 = Math.min(selectionStart.value.x, selectionEnd.value.x)
  const y1 = Math.min(selectionStart.value.y, selectionEnd.value.y)
  const x2 = Math.max(selectionStart.value.x, selectionEnd.value.x)
  const y2 = Math.max(selectionStart.value.y, selectionEnd.value.y)

  desktopStore.clearSelection()

  desktopItems.value.forEach((item) => {
    // Check if item position is within selection box
    const itemX = item.positionX + 60 // Icon center (approx)
    const itemY = item.positionY + 60

    if (itemX >= x1 && itemX <= x2 && itemY >= y1 && itemY <= y2) {
      desktopStore.toggleSelection(item.id, true) // true = add to selection
    }
  })
}

// Swiper event handlers
const onSwiperInit = (swiper: SwiperType) => {
  swiperInstance.value = swiper
}

const onSlideChange = (swiper: SwiperType) => {
  workspaceStore.switchToWorkspace(
    workspaceStore.workspaces.at(swiper.activeIndex)?.id,
  )
}

// Workspace control handlers
const handleAddWorkspace = async () => {
  await workspaceStore.addWorkspaceAsync()
  // Swiper will auto-slide to new workspace because we switch in addWorkspaceAsync
  nextTick(() => {
    if (swiperInstance.value) {
      swiperInstance.value.slideTo(workspaces.value.length - 1)
    }
  })
}

/* const handleRemoveWorkspace = async () => {
  if (!currentWorkspace.value || workspaces.value.length <= 1) return

  const currentIndex = currentWorkspaceIndex.value
  await workspaceStore.removeWorkspaceAsync(currentWorkspace.value.id)

  // Slide to adjusted index
  nextTick(() => {
    if (swiperInstance.value) {
      const newIndex = Math.min(currentIndex, workspaces.value.length - 1)
      swiperInstance.value.slideTo(newIndex)
    }
  })
}

const handleDropWindowOnWorkspace = async (
  event: DragEvent,
  targetWorkspaceId: string,
) => {
  // Get the window ID from drag data (will be set when we implement window dragging)
  const windowId = event.dataTransfer?.getData('windowId')
  if (windowId) {
    await moveWindowToWorkspace(windowId, targetWorkspaceId)
  }
} */

// Overview Mode: Calculate grid positions and scale for windows
// Calculate preview dimensions for window overview
const MIN_PREVIEW_WIDTH = 300 // 50% increase from 200
const MAX_PREVIEW_WIDTH = 600 // 50% increase from 400
const MIN_PREVIEW_HEIGHT = 225 // 50% increase from 150
const MAX_PREVIEW_HEIGHT = 450 // 50% increase from 300

// Store window state for overview (position only, size stays original)
const overviewWindowState = ref(
  new Map<string, { x: number; y: number; width: number; height: number; scale: number }>(),
)

// Calculate scale and card dimensions for each window
watch(
  () => windowManager.showWindowOverview,
  (isOpen) => {
    if (isOpen) {
      // Wait for the Overview modal to mount and create the teleport targets
      nextTick(() => {
        windowManager.windows.forEach((window) => {
          const scaleX = MAX_PREVIEW_WIDTH / window.width
          const scaleY = MAX_PREVIEW_HEIGHT / window.height
          const scale = Math.min(scaleX, scaleY, 1)

          // Ensure minimum card size
          const scaledWidth = window.width * scale
          const scaledHeight = window.height * scale

          let finalScale = scale
          if (scaledWidth < MIN_PREVIEW_WIDTH) {
            finalScale = MIN_PREVIEW_WIDTH / window.width
          }
          if (scaledHeight < MIN_PREVIEW_HEIGHT) {
            finalScale = Math.max(finalScale, MIN_PREVIEW_HEIGHT / window.height)
          }

          overviewWindowState.value.set(window.id, {
            x: 0,
            y: 0,
            width: window.width,
            height: window.height,
            scale: finalScale,
          })
        })
      })
    } else {
      // Clear state when overview is closed
      overviewWindowState.value.clear()
    }
  },
)

// Disable Swiper in overview mode
watch(isOverviewMode, (newValue) => {
  allowSwipe.value = !newValue
})

// Watch for workspace changes to reload desktop items
watch(currentWorkspace, async () => {
  if (currentWorkspace.value) {
    await desktopStore.loadDesktopItemsAsync()
  }
})

onMounted(async () => {
  // Load workspaces first
  await workspaceStore.loadWorkspacesAsync()

  // Then load desktop items for current workspace
  await desktopStore.loadDesktopItemsAsync()
})
</script>

<style scoped>
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.3s ease;
}

.slide-down-enter-from {
  opacity: 0;
  transform: translateY(-100%);
}

.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-100%);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

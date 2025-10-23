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
      :threshold="10"
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
            v-for="(window, index) in getWorkspaceWindows(workspace.id)"
            :key="window.id"
          >
            <!-- Wrapper for Overview Mode Click/Drag -->
            <div
              v-if="false"
              :style="
                getOverviewWindowGridStyle(
                  index,
                  getWorkspaceWindows(workspace.id).length,
                )
              "
              class="absolute cursor-pointer group"
              :draggable="true"
              @dragstart="handleOverviewWindowDragStart($event, window.id)"
              @dragend="handleOverviewWindowDragEnd"
              @click="handleOverviewWindowClick(window.id)"
            >
              <!-- Overlay for click/drag events (prevents interaction with window content) -->
              <div
                class="absolute inset-0 z-[100] bg-transparent group-hover:ring-4 group-hover:ring-purple-500 rounded-xl transition-all"
              />

              <HaexWindow
                :id="window.id"
                :title="window.title"
                :icon="window.icon"
                :initial-x="window.x"
                :initial-y="window.y"
                :initial-width="window.width"
                :initial-height="window.height"
                :is-active="windowManager.isWindowActive(window.id)"
                :source-x="window.sourceX"
                :source-y="window.sourceY"
                :source-width="window.sourceWidth"
                :source-height="window.sourceHeight"
                :is-opening="window.isOpening"
                :is-closing="window.isClosing"
                class="no-swipe pointer-events-none"
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
                {{ window }}
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

            <!-- Normal Mode (non-overview) -->
            <HaexWindow
              :id="window.id"
              :title="window.title"
              :icon="window.icon"
              :initial-x="window.x"
              :initial-y="window.y"
              :initial-width="window.width"
              :initial-height="window.height"
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
          </template>
        </div>
      </SwiperSlide>
    </Swiper>

    <!-- Workspace Drawer -->
    <UDrawer
      v-model:open="isOverviewMode"
      direction="left"
      :dismissible="false"
      :overlay="false"
      :modal="false"
      should-scale-background
      set-background-color-on-scale
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

import { eq } from 'drizzle-orm'
import { haexDesktopItems } from '~~/src-tauri/database/schemas'

const SwiperNavigation = Navigation

const desktopStore = useDesktopStore()
const extensionsStore = useExtensionsStore()
const windowManager = useWindowManagerStore()
const workspaceStore = useWorkspaceStore()

const { currentVault } = storeToRefs(useVaultStore())
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
const currentDraggingWindowId = ref<string | null>(null)
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

// Dropzone refs
/* const removeDropzoneEl = ref<HTMLElement>()
const uninstallDropzoneEl = ref<HTMLElement>() */

// Setup dropzones with VueUse
/* const { isOverDropZone: isOverRemoveZone } = useDropZone(removeDropzoneEl, {
  onDrop: () => {
    if (currentDraggedItemId.value) {
      handleRemoveFromDesktop(currentDraggedItemId.value)
    }
  },
}) */

/* const { isOverDropZone: isOverUninstallZone } = useDropZone(uninstallDropzoneEl, {
  onDrop: () => {
    if (currentDraggedItemType.value && currentDraggedReferenceId.value) {
      handleUninstall(currentDraggedItemType.value, currentDraggedReferenceId.value)
    }
  },
}) */

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

// Get windows for a specific workspace
const getWorkspaceWindows = (workspaceId: string) => {
  return windowManager.windows.filter(
    (w) => w.workspaceId === workspaceId && !w.isMinimized,
  )
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

// Move desktop item to different workspace
const moveItemToWorkspace = async (
  itemId: string,
  targetWorkspaceId: string,
) => {
  const item = desktopItems.value.find((i) => i.id === itemId)
  if (!item) return

  try {
    if (!currentVault.value?.drizzle) return

    await currentVault.value.drizzle
      .update(haexDesktopItems)
      .set({ workspaceId: targetWorkspaceId })
      .where(eq(haexDesktopItems.id, itemId))

    // Update local state
    item.workspaceId = targetWorkspaceId
  } catch (error) {
    console.error('Fehler beim Verschieben des Items:', error)
  }
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
  isWindowDragging.value = true
  currentDraggingWindowId.value = windowId
  allowSwipe.value = false // Disable Swiper during window drag
}

const handleWindowDragEnd = async () => {
  // Window handles snapping itself, we just need to cleanup state
  isWindowDragging.value = false
  currentDraggingWindowId.value = null
  allowSwipe.value = true // Re-enable Swiper after drag
}

// Move window to different workspace
const moveWindowToWorkspace = async (
  windowId: string,
  targetWorkspaceId: string,
) => {
  const window = windowManager.windows.find((w) => w.id === windowId)
  if (!window) return

  // Update window's workspaceId
  window.workspaceId = targetWorkspaceId
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

const handleSwitchToWorkspace = (index: number) => {
  if (swiperInstance.value) {
    swiperInstance.value.slideTo(index)
  }
}

const handleRemoveWorkspace = async () => {
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

// Drawer handlers
const handleSwitchToWorkspaceFromDrawer = (index: number) => {
  handleSwitchToWorkspace(index)
  // Close drawer after switch
  isOverviewMode.value = false
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
}

// Overview Mode: Calculate grid positions and scale for windows
const getOverviewWindowGridStyle = (index: number, totalWindows: number) => {
  if (!viewportWidth.value || !viewportHeight.value) {
    return {}
  }

  // Determine grid layout based on number of windows
  let cols = 1
  let rows = 1

  if (totalWindows === 1) {
    cols = 1
    rows = 1
  } else if (totalWindows === 2) {
    cols = 2
    rows = 1
  } else if (totalWindows <= 4) {
    cols = 2
    rows = 2
  } else if (totalWindows <= 6) {
    cols = 3
    rows = 2
  } else if (totalWindows <= 9) {
    cols = 3
    rows = 3
  } else {
    cols = 4
    rows = Math.ceil(totalWindows / 4)
  }

  // Calculate grid cell position
  const col = index % cols
  const row = Math.floor(index / cols)

  // Padding and gap
  const padding = 40 // px from viewport edges
  const gap = 30 // px between windows

  // Available space
  const availableWidth = viewportWidth.value - padding * 2 - gap * (cols - 1)
  const availableHeight = viewportHeight.value - padding * 2 - gap * (rows - 1)

  // Cell dimensions
  const cellWidth = availableWidth / cols
  const cellHeight = availableHeight / rows

  // Window aspect ratio (assume 16:9 or use actual window dimensions)
  const windowAspectRatio = 16 / 9

  // Calculate scale to fit window in cell
  const targetWidth = cellWidth
  const targetHeight = cellHeight
  const targetAspect = targetWidth / targetHeight

  let scale = 0.25 // Default scale
  let scaledWidth = 800 * scale
  let scaledHeight = 600 * scale

  if (targetAspect > windowAspectRatio) {
    // Cell is wider than window aspect ratio - fit by height
    scaledHeight = Math.min(targetHeight, 600 * 0.4)
    scale = scaledHeight / 600
    scaledWidth = 800 * scale
  } else {
    // Cell is taller than window aspect ratio - fit by width
    scaledWidth = Math.min(targetWidth, 800 * 0.4)
    scale = scaledWidth / 800
    scaledHeight = 600 * scale
  }

  // Calculate position to center window in cell
  const cellX = padding + col * (cellWidth + gap)
  const cellY = padding + row * (cellHeight + gap)

  // Center window in cell
  const x = cellX + (cellWidth - scaledWidth) / 2
  const y = cellY + (cellHeight - scaledHeight) / 2

  return {
    transform: `scale(${scale})`,
    transformOrigin: 'top left',
    left: `${x / scale}px`,
    top: `${y / scale}px`,
    width: '800px',
    height: '600px',
    zIndex: 91,
    transition: 'all 0.3s ease',
  }
}

// Overview Mode handlers
const handleOverviewWindowClick = (windowId: string) => {
  // Activate the window
  windowManager.activateWindow(windowId)
  // Close overview mode
  isOverviewMode.value = false
}

const handleOverviewWindowDragStart = (event: DragEvent, windowId: string) => {
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('windowId', windowId)
  }
}

const handleOverviewWindowDragEnd = () => {
  // Cleanup after drag
}

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

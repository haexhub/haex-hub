<template>
  <div
    ref="windowEl"
    :style="windowStyle"
    :class="[
      'absolute bg-white/80 dark:bg-gray-900/80 backdrop-blur-xl rounded-xl shadow-2xl overflow-hidden',
      'border border-gray-200 dark:border-gray-700 transition-all ease-out duration-600',
      'flex flex-col',
      isActive ? 'z-50' : 'z-10',
    ]"
    @mousedown="handleActivate"
  >
    <!-- Window Titlebar -->
    <div
      ref="titlebarEl"
      class="grid grid-cols-3 items-center px-3 py-1 bg-white/80 dark:bg-gray-800/80 border-b border-gray-200/50 dark:border-gray-700/50 cursor-move select-none touch-none"
      @dblclick="handleMaximize"
    >
      <!-- Left: Icon -->
      <div class="flex items-center gap-2">
        <img
          v-if="icon"
          :src="icon"
          :alt="title"
          class="w-5 h-5 object-contain flex-shrink-0"
        />
      </div>

      <!-- Center: Title -->
      <div class="flex items-center justify-center">
        <span
          class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate max-w-full"
        >
          {{ title }}
        </span>
      </div>

      <!-- Right: Window Controls -->
      <div class="flex items-center gap-1 justify-end">
        <button
          class="w-8 h-8 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 flex items-center justify-center transition-colors"
          @click.stop="handleMinimize"
        >
          <UIcon
            name="i-heroicons-minus"
            class="w-4 h-4 text-gray-600 dark:text-gray-400"
          />
        </button>
        <button
          class="w-8 h-8 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 flex items-center justify-center transition-colors"
          @click.stop="handleMaximize"
        >
          <UIcon
            :name="
              isMaximized
                ? 'i-heroicons-arrows-pointing-in'
                : 'i-heroicons-arrows-pointing-out'
            "
            class="w-4 h-4 text-gray-600 dark:text-gray-400"
          />
        </button>
        <button
          class="w-8 h-8 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 flex items-center justify-center transition-colors group"
          @click.stop="handleClose"
        >
          <UIcon
            name="i-heroicons-x-mark"
            class="w-4 h-4 text-gray-600 dark:text-gray-400 group-hover:text-red-600 dark:group-hover:text-red-400"
          />
        </button>
      </div>
    </div>

    <!-- Window Content -->
    <div
      :class="[
        'flex-1 overflow-hidden relative',
        isDragging || isResizing ? 'pointer-events-none' : '',
      ]"
    >
      <slot />
    </div>

    <!-- Resize Handles -->
    <template v-if="!isMaximized">
      <div
        class="absolute top-0 left-0 w-2 h-2 cursor-nw-resize"
        @mousedown.left.stop="handleResizeStart('nw', $event)"
      />
      <div
        class="absolute top-0 right-0 w-2 h-2 cursor-ne-resize"
        @mousedown.left.stop="handleResizeStart('ne', $event)"
      />
      <div
        class="absolute bottom-0 left-0 w-2 h-2 cursor-sw-resize"
        @mousedown.left.stop="handleResizeStart('sw', $event)"
      />
      <div
        class="absolute bottom-0 right-0 w-2 h-2 cursor-se-resize"
        @mousedown.left.stop="handleResizeStart('se', $event)"
      />
      <div
        class="absolute top-0 left-2 right-2 h-1 cursor-n-resize"
        @mousedown.left.stop="handleResizeStart('n', $event)"
      />
      <div
        class="absolute bottom-0 left-2 right-2 h-1 cursor-s-resize"
        @mousedown.left.stop="handleResizeStart('s', $event)"
      />
      <div
        class="absolute left-0 top-2 bottom-2 w-1 cursor-w-resize"
        @mousedown.left.stop="handleResizeStart('w', $event)"
      />
      <div
        class="absolute right-0 top-2 bottom-2 w-1 cursor-e-resize"
        @mousedown.left.stop="handleResizeStart('e', $event)"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  id: string
  title: string
  icon?: string
  initialX?: number
  initialY?: number
  initialWidth?: number
  initialHeight?: number
  isActive?: boolean
  sourceX?: number
  sourceY?: number
  sourceWidth?: number
  sourceHeight?: number
  isOpening?: boolean
  isClosing?: boolean
}>()

const emit = defineEmits<{
  close: []
  minimize: []
  activate: []
  positionChanged: [x: number, y: number]
  sizeChanged: [width: number, height: number]
  dragStart: []
  dragEnd: []
}>()

const windowEl = ref<HTMLElement>()
const titlebarEl = useTemplateRef('titlebarEl')

// Inject viewport size from parent desktop
const viewportSize = inject<{
  width: Ref<number>
  height: Ref<number>
}>('viewportSize')

// Window state
const x = ref(props.initialX ?? 100)
const y = ref(props.initialY ?? 100)
const width = ref(props.initialWidth ?? 800)
const height = ref(props.initialHeight ?? 600)
const isMaximized = ref(false) // Don't start maximized

// Store initial position/size for restore
const preMaximizeState = ref({
  x: props.initialX ?? 100,
  y: props.initialY ?? 100,
  width: props.initialWidth ?? 800,
  height: props.initialHeight ?? 600,
})

// Dragging state
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)

// Resizing state
const isResizing = ref(false)
const resizeDirection = ref<string>('')
const resizeStartX = ref(0)
const resizeStartY = ref(0)
const resizeStartWidth = ref(0)
const resizeStartHeight = ref(0)
const resizeStartPosX = ref(0)
const resizeStartPosY = ref(0)

// Snap settings
const snapEdgeThreshold = 50 // pixels from edge to trigger snap
const { x: mouseX } = useMouse()

// Setup drag with useDrag composable (supports mouse + touch)
useDrag(
  ({ movement: [mx, my], first, last }) => {
    if (isMaximized.value) return

    if (first) {
      // Drag started - save initial position
      isDragging.value = true
      dragStartX.value = x.value
      dragStartY.value = y.value
      emit('dragStart')
      return // Don't update position on first event
    }

    if (last) {
      // Drag ended - apply snapping
      isDragging.value = false

      const viewportBounds = getViewportBounds()
      if (viewportBounds) {
        const viewportWidth = viewportBounds.width
        const viewportHeight = viewportBounds.height

        if (mouseX.value <= snapEdgeThreshold) {
          // Snap to left half
          x.value = 0
          y.value = 0
          width.value = viewportWidth / 2
          height.value = viewportHeight
          isMaximized.value = false
        } else if (mouseX.value >= viewportWidth - snapEdgeThreshold) {
          // Snap to right half
          x.value = viewportWidth / 2
          y.value = 0
          width.value = viewportWidth / 2
          height.value = viewportHeight
          isMaximized.value = false
        } else {
          // Normal snap back to viewport
          snapToViewport()
        }
      }

      emit('positionChanged', x.value, y.value)
      emit('sizeChanged', width.value, height.value)
      emit('dragEnd')
      return
    }

    // Dragging (not first, not last)
    const newX = dragStartX.value + mx
    const newY = dragStartY.value + my

    // Apply constraints during drag
    const constrained = constrainToViewportDuringDrag(newX, newY)
    x.value = constrained.x
    y.value = constrained.y
  },
  {
    domTarget: titlebarEl,
    eventOptions: { passive: false },
    pointer: { touch: true },
    drag: {
      threshold: 10, // 10px threshold prevents accidental drags and improves performance
      filterTaps: true, // Filter out taps (clicks) vs drags
      delay: 0, // No delay for immediate response
    },
  },
)

const windowStyle = computed(() => {
  const baseStyle: Record<string, string> = {}

  // Opening animation: start from icon position
  if (
    props.isOpening &&
    props.sourceX !== undefined &&
    props.sourceY !== undefined
  ) {
    baseStyle.left = `${props.sourceX}px`
    baseStyle.top = `${props.sourceY}px`
    baseStyle.width = `${props.sourceWidth || 100}px`
    baseStyle.height = `${props.sourceHeight || 100}px`
    baseStyle.opacity = '0'
    baseStyle.transform = 'scale(0.3)'
  }
  // Closing animation: shrink to icon position
  else if (
    props.isClosing &&
    props.sourceX !== undefined &&
    props.sourceY !== undefined
  ) {
    baseStyle.left = `${props.sourceX}px`
    baseStyle.top = `${props.sourceY}px`
    baseStyle.width = `${props.sourceWidth || 100}px`
    baseStyle.height = `${props.sourceHeight || 100}px`
    baseStyle.opacity = '0'
    baseStyle.transform = 'scale(0.3)'
  }
  // Normal state
  else if (isMaximized.value) {
    baseStyle.left = '0px'
    baseStyle.top = '0px'
    baseStyle.width = '100%'
    baseStyle.height = '100%'
    baseStyle.borderRadius = '0'
    baseStyle.opacity = '1'
    baseStyle.transform = 'scale(1)'
  } else {
    baseStyle.left = `${x.value}px`
    baseStyle.top = `${y.value}px`
    baseStyle.width = `${width.value}px`
    baseStyle.height = `${height.value}px`
    baseStyle.opacity = '1'
    baseStyle.transform = 'scale(1)'
  }

  // Performance optimization: hint browser about transforms
  if (isDragging.value || isResizing.value) {
    baseStyle.willChange = 'transform, width, height'
  }

  return baseStyle
})

const getViewportBounds = () => {
  // Use reactive viewport size from parent if available
  if (viewportSize) {
    return {
      width: viewportSize.width.value,
      height: viewportSize.height.value,
    }
  }

  // Fallback to parent element measurement
  if (!windowEl.value?.parentElement) return null

  const parent = windowEl.value.parentElement
  return {
    width: parent.clientWidth,
    height: parent.clientHeight,
  }
}

const constrainToViewportDuringDrag = (newX: number, newY: number) => {
  const bounds = getViewportBounds()
  if (!bounds) return { x: newX, y: newY }

  const windowWidth = width.value
  const windowHeight = height.value

  // Allow max 1/3 of window to go outside viewport during drag
  const maxOffscreenX = windowWidth / 3
  const maxOffscreenY = windowHeight / 3

  const maxX = bounds.width - windowWidth + maxOffscreenX
  const minX = -maxOffscreenX
  const maxY = bounds.height - windowHeight + maxOffscreenY
  const minY = -maxOffscreenY

  const constrainedX = Math.max(minX, Math.min(maxX, newX))
  const constrainedY = Math.max(minY, Math.min(maxY, newY))

  return { x: constrainedX, y: constrainedY }
}

const constrainToViewportFully = (
  newX: number,
  newY: number,
  newWidth?: number,
  newHeight?: number,
) => {
  const bounds = getViewportBounds()
  if (!bounds) return { x: newX, y: newY }

  const windowWidth = newWidth ?? width.value
  const windowHeight = newHeight ?? height.value

  // Keep entire window within viewport
  const maxX = bounds.width - windowWidth
  const minX = 0
  const maxY = bounds.height - windowHeight
  const minY = 0

  const constrainedX = Math.max(minX, Math.min(maxX, newX))
  const constrainedY = Math.max(minY, Math.min(maxY, newY))

  return { x: constrainedX, y: constrainedY }
}

const snapToViewport = () => {
  const bounds = getViewportBounds()
  if (!bounds) return

  const constrained = constrainToViewportFully(x.value, y.value)
  x.value = constrained.x
  y.value = constrained.y
}

const handleActivate = () => {
  emit('activate')
}

const handleClose = () => {
  emit('close')
}

const handleMinimize = () => {
  emit('minimize')
}

const handleMaximize = () => {
  if (isMaximized.value) {
    // Restore
    x.value = preMaximizeState.value.x
    y.value = preMaximizeState.value.y
    width.value = preMaximizeState.value.width
    height.value = preMaximizeState.value.height
    isMaximized.value = false
  } else {
    // Maximize
    preMaximizeState.value = {
      x: x.value,
      y: y.value,
      width: width.value,
      height: height.value,
    }
    isMaximized.value = true
  }
}

// Window resizing
const handleResizeStart = (direction: string, e: MouseEvent) => {
  isResizing.value = true
  resizeDirection.value = direction
  resizeStartX.value = e.clientX
  resizeStartY.value = e.clientY
  resizeStartWidth.value = width.value
  resizeStartHeight.value = height.value
  resizeStartPosX.value = x.value
  resizeStartPosY.value = y.value
}

// Global mouse move handler (for resizing only, dragging handled by useDrag)
useEventListener(window, 'mousemove', (e: MouseEvent) => {
  if (isResizing.value) {
    const deltaX = e.clientX - resizeStartX.value
    const deltaY = e.clientY - resizeStartY.value

    const dir = resizeDirection.value

    // Handle width changes
    if (dir.includes('e')) {
      width.value = Math.max(300, resizeStartWidth.value + deltaX)
    } else if (dir.includes('w')) {
      const newWidth = Math.max(300, resizeStartWidth.value - deltaX)
      const widthDiff = resizeStartWidth.value - newWidth
      x.value = resizeStartPosX.value + widthDiff
      width.value = newWidth
    }

    // Handle height changes
    if (dir.includes('s')) {
      height.value = Math.max(200, resizeStartHeight.value + deltaY)
    } else if (dir.includes('n')) {
      const newHeight = Math.max(200, resizeStartHeight.value - deltaY)
      const heightDiff = resizeStartHeight.value - newHeight
      y.value = resizeStartPosY.value + heightDiff
      height.value = newHeight
    }
  }
})

// Global mouse up handler (for resizing only, dragging handled by useDrag)
useEventListener(window, 'mouseup', () => {
  if (isResizing.value) {
    isResizing.value = false

    // Snap back to viewport after resize ends
    snapToViewport()

    emit('positionChanged', x.value, y.value)
    emit('sizeChanged', width.value, height.value)
  }
})
</script>

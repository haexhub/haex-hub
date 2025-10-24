<template>
  <div
    ref="windowEl"
    :style="windowStyle"
    :class="[
      'absolute bg-default/80 backdrop-blur-xl rounded-xl shadow-2xl overflow-hidden isolate',
      'border border-gray-200 dark:border-gray-700 transition-all ease-out duration-600 ',
      'flex flex-col @container',
      { 'select-none': isResizingOrDragging },
      isActive ? 'z-100' : 'z-50',
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
        'flex-1 overflow-auto relative ',
        isResizingOrDragging ? 'pointer-events-none' : '',
      ]"
    >
      <slot />
    </div>

    <!-- Resize Handles -->
    <HaexWindowResizeHandles
      :disabled="isMaximized"
      @resize-start="handleResizeStart"
    />
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  id: string
  title: string
  icon?: string | null
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

// Use defineModel for x, y, width, height
const x = defineModel<number>('x', { default: 100 })
const y = defineModel<number>('y', { default: 100 })
const width = defineModel<number>('width', { default: 800 })
const height = defineModel<number>('height', { default: 600 })

const windowEl = useTemplateRef('windowEl')
const titlebarEl = useTemplateRef('titlebarEl')

// Inject viewport size from parent desktop
const viewportSize = inject<{
  width: Ref<number>
  height: Ref<number>
}>('viewportSize')
const isMaximized = ref(false) // Don't start maximized

// Store initial position/size for restore
const preMaximizeState = ref({
  x: x.value,
  y: y.value,
  width: width.value,
  height: height.value,
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

const isResizingOrDragging = computed(
  () => isResizing.value || isDragging.value,
)

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
      // Drag ended
      isDragging.value = false
      globalThis.getSelection()?.removeAllRanges()
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
  // Normal state (maximized windows now use actual pixel dimensions)
  else {
    baseStyle.left = `${x.value}px`
    baseStyle.top = `${y.value}px`
    baseStyle.width = `${width.value}px`
    baseStyle.height = `${height.value}px`
    baseStyle.opacity = '1'

    // Remove border-radius when maximized
    if (isMaximized.value) {
      baseStyle.borderRadius = '0'
    }
  }

  // Performance optimization: hint browser about transforms
  if (isDragging.value || isResizing.value) {
    baseStyle.willChange = 'transform, width, height'
    baseStyle.transform = 'translateZ(0)'
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

  // Allow sides and bottom to go out more
  const maxOffscreenX = windowWidth / 3
  const maxOffscreenBottom = windowHeight / 3

  // For X axis: allow 1/3 to go outside on both sides
  const maxX = bounds.width - windowWidth + maxOffscreenX
  const minX = -maxOffscreenX

  // For Y axis: HARD constraint at top (y=0), never allow window to go above header
  const minY = 0
  // Bottom: allow 1/3 to go outside
  const maxY = bounds.height - windowHeight + maxOffscreenBottom

  const constrainedX = Math.max(minX, Math.min(maxX, newX))
  const constrainedY = Math.max(minY, Math.min(maxY, newY))

  return { x: constrainedX, y: constrainedY }
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
    // Maximize - set position and size to viewport dimensions
    preMaximizeState.value = {
      x: x.value,
      y: y.value,
      width: width.value,
      height: height.value,
    }

    // Get viewport bounds (desktop container, already excludes header)
    const bounds = getViewportBounds()

    if (bounds && bounds.width > 0 && bounds.height > 0) {
      x.value = 0
      y.value = 0
      width.value = bounds.width
      height.value = bounds.height
      isMaximized.value = true
    }
    console.log('handleMaximize', preMaximizeState, bounds)
  }
}

// Window resizing
const handleResizeStart = (direction: string, e: MouseEvent | TouchEvent) => {
  isResizing.value = true
  resizeDirection.value = direction
  let clientX: number
  let clientY: number

  if ('touches' in e) {
    // Es ist ein TouchEvent
    const touch = e.touches[0] // Hole den ersten Touch

    // Prüfe, ob 'touch' existiert (ist undefined, wenn e.touches leer ist)
    if (touch) {
      clientX = touch.clientX
      clientY = touch.clientY
    } else {
      // Ungültiges Start-Event (kein Finger). Abbruch.
      isResizing.value = false
      return
    }
  } else {
    // Es ist ein MouseEvent
    clientX = e.clientX
    clientY = e.clientY
  }

  resizeStartX.value = clientX
  resizeStartY.value = clientY
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
    globalThis.getSelection()?.removeAllRanges()
    isResizing.value = false

    emit('positionChanged', x.value, y.value)
    emit('sizeChanged', width.value, height.value)
  }
})
</script>

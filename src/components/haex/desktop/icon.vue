<template>
  <UContextMenu :items="contextMenuItems">
    <div
      ref="draggableEl"
      :style="style"
      class="select-none cursor-grab active:cursor-grabbing"
      @pointerdown="handlePointerDown"
      @pointermove="handlePointerMove"
      @pointerup="handlePointerUp"
      @dblclick="handleOpen"
    >
      <div class="flex flex-col items-center gap-1 p-2">
        <div
          class="w-16 h-16 flex items-center justify-center bg-white/90 dark:bg-gray-800/90 rounded-lg shadow-lg hover:shadow-xl transition-shadow"
        >
          <img v-if="icon" :src="icon" :alt="label" class="w-12 h-12 object-contain" />
          <Icon v-else name="i-heroicons-puzzle-piece-solid" class="w-12 h-12 text-gray-500" />
        </div>
        <span
          class="text-xs text-center max-w-20 truncate bg-white/80 dark:bg-gray-800/80 px-2 py-1 rounded shadow"
        >
          {{ label }}
        </span>
      </div>
    </div>
  </UContextMenu>
</template>

<script setup lang="ts">
const props = defineProps<{
  id: string
  itemType: 'extension' | 'file' | 'folder'
  referenceId: string
  initialX: number
  initialY: number
  label: string
  icon?: string
}>()

const emit = defineEmits<{
  positionChanged: [id: string, x: number, y: number]
  open: [itemType: string, referenceId: string]
  uninstall: [itemType: string, referenceId: string]
  dragStart: [id: string, itemType: string, referenceId: string]
  dragEnd: []
}>()

const desktopStore = useDesktopStore()

const contextMenuItems = computed(() =>
  desktopStore.getContextMenuItems(
    props.id,
    props.itemType,
    props.referenceId,
    handleOpen,
    handleUninstall,
  ),
)

const draggableEl = ref<HTMLElement>()
const x = ref(props.initialX)
const y = ref(props.initialY)
const isDragging = ref(false)
const offsetX = ref(0)
const offsetY = ref(0)

const style = computed(() => ({
  position: 'absolute' as const,
  left: `${x.value}px`,
  top: `${y.value}px`,
  touchAction: 'none' as const,
}))

const handlePointerDown = (e: PointerEvent) => {
  if (!draggableEl.value || !draggableEl.value.parentElement) return

  isDragging.value = true
  emit('dragStart', props.id, props.itemType, props.referenceId)

  // Get parent offset to convert from viewport coordinates to parent-relative coordinates
  const parentRect = draggableEl.value.parentElement.getBoundingClientRect()

  // Calculate offset from mouse position to current element position (in parent coordinates)
  offsetX.value = e.clientX - parentRect.left - x.value
  offsetY.value = e.clientY - parentRect.top - y.value

  draggableEl.value.setPointerCapture(e.pointerId)
}

const handlePointerMove = (e: PointerEvent) => {
  if (!isDragging.value || !draggableEl.value?.parentElement) return

  const parentRect = draggableEl.value.parentElement.getBoundingClientRect()
  const newX = e.clientX - parentRect.left - offsetX.value
  const newY = e.clientY - parentRect.top - offsetY.value

  x.value = newX
  y.value = newY
}

const handlePointerUp = (e: PointerEvent) => {
  if (!isDragging.value) return

  isDragging.value = false
  if (draggableEl.value) {
    draggableEl.value.releasePointerCapture(e.pointerId)
  }
  emit('dragEnd')
  emit('positionChanged', props.id, x.value, y.value)
}

const handleOpen = () => {
  emit('open', props.itemType, props.referenceId)
}

const handleUninstall = () => {
  emit('uninstall', props.itemType, props.referenceId)
}
</script>

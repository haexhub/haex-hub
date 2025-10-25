<template>
  <div>
    <UiDialogConfirm
      v-model:open="showUninstallDialog"
      :title="t('confirmUninstall.title')"
      :description="t('confirmUninstall.message', { name: label })"
      :confirm-label="t('confirmUninstall.confirm')"
      :abort-label="t('confirmUninstall.cancel')"
      confirm-icon="i-heroicons-trash"
      @confirm="handleConfirmUninstall"
    />

    <UContextMenu :items="contextMenuItems">
      <div
        ref="draggableEl"
        :style="style"
        class="select-none cursor-grab active:cursor-grabbing"
        @pointerdown.left="handlePointerDown"
        @pointermove="handlePointerMove"
        @pointerup="handlePointerUp"
        @click.left="handleClick"
        @dblclick="handleDoubleClick"
      >
        <div class="flex flex-col items-center gap-2 p-3 group">
          <div
            :class="[
              'w-20 h-20 flex items-center justify-center rounded-2xl transition-all duration-200 ease-out',
              'backdrop-blur-sm border',
              isSelected
                ? 'bg-white/95 dark:bg-gray-800/95 border-blue-500 dark:border-blue-400 shadow-lg scale-105'
                : 'bg-white/80 dark:bg-gray-800/80 border-gray-200/50 dark:border-gray-700/50 hover:bg-white/90 dark:hover:bg-gray-800/90 hover:border-gray-300 dark:hover:border-gray-600 hover:shadow-md hover:scale-105',
            ]"
          >
            <img
              v-if="icon"
              :src="icon"
              :alt="label"
              class="w-14 h-14 object-contain transition-transform duration-200"
              :class="{ 'scale-110': isSelected }"
            />
            <UIcon
              v-else
              name="i-heroicons-puzzle-piece-solid"
              :class="[
                'w-14 h-14 transition-all duration-200',
                isSelected
                  ? 'text-blue-500 dark:text-blue-400 scale-110'
                  : 'text-gray-400 dark:text-gray-500 group-hover:text-gray-500 dark:group-hover:text-gray-400',
              ]"
            />
          </div>
          <span
            :class="[
              'text-xs text-center max-w-24 truncate px-3 py-1.5 rounded-lg transition-all duration-200',
              'backdrop-blur-sm',
              isSelected
                ? 'bg-white/95 dark:bg-gray-800/95 text-gray-900 dark:text-gray-100 font-medium shadow-md'
                : 'bg-white/70 dark:bg-gray-800/70 text-gray-700 dark:text-gray-300 group-hover:bg-white/85 dark:group-hover:bg-gray-800/85',
            ]"
          >
            {{ label }}
          </span>
        </div>
      </div>
    </UContextMenu>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  id: string
  itemType: DesktopItemType
  referenceId: string
  initialX: number
  initialY: number
  label: string
  icon?: string
}>()

const emit = defineEmits<{
  positionChanged: [id: string, x: number, y: number]
  dragStart: [id: string, itemType: string, referenceId: string]
  dragEnd: []
}>()

const desktopStore = useDesktopStore()
const showUninstallDialog = ref(false)
const { t } = useI18n()

const isSelected = computed(() => desktopStore.isItemSelected(props.id))

const handleClick = (e: MouseEvent) => {
  // Prevent selection during drag
  if (isDragging.value) return

  desktopStore.toggleSelection(props.id, e.ctrlKey || e.metaKey)
}

const handleUninstallClick = () => {
  showUninstallDialog.value = true
}

const handleConfirmUninstall = async () => {
  showUninstallDialog.value = false
  await desktopStore.uninstallDesktopItem(
    props.id,
    props.itemType,
    props.referenceId,
  )
}

const contextMenuItems = computed(() =>
  desktopStore.getContextMenuItems(
    props.id,
    props.itemType,
    props.referenceId,
    handleUninstallClick,
  ),
)

// Inject viewport size from parent desktop
const viewportSize = inject<{
  width: Ref<number>
  height: Ref<number>
}>('viewportSize')

const draggableEl = ref<HTMLElement>()
const x = ref(props.initialX)
const y = ref(props.initialY)
const isDragging = ref(false)
const offsetX = ref(0)
const offsetY = ref(0)

// Icon dimensions (approximate)
const iconWidth = 120 // Matches design in template
const iconHeight = 140

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

  // Snap icon to viewport bounds if outside
  if (viewportSize) {
    const maxX = Math.max(0, viewportSize.width.value - iconWidth)
    const maxY = Math.max(0, viewportSize.height.value - iconHeight)
    x.value = Math.max(0, Math.min(maxX, x.value))
    y.value = Math.max(0, Math.min(maxY, y.value))
  }

  emit('dragEnd')
  emit('positionChanged', props.id, x.value, y.value)
}

const handleDoubleClick = () => {
  // Get icon position and size for animation
  if (draggableEl.value) {
    const rect = draggableEl.value.getBoundingClientRect()
    const sourcePosition = {
      x: rect.left,
      y: rect.top,
      width: rect.width,
      height: rect.height,
    }
    desktopStore.openDesktopItem(
      props.itemType,
      props.referenceId,
      sourcePosition,
    )
  } else {
    desktopStore.openDesktopItem(props.itemType, props.referenceId)
  }
}
</script>

<i18n lang="yaml">
de:
  confirmUninstall:
    title: Erweiterung deinstallieren
    message: Möchten Sie die Erweiterung '{name}' wirklich deinstallieren? Diese Aktion kann nicht rückgängig gemacht werden.
    confirm: Deinstallieren
    cancel: Abbrechen

en:
  confirmUninstall:
    title: Uninstall Extension
    message: Do you really want to uninstall the extension '{name}'? This action cannot be undone.
    confirm: Uninstall
    cancel: Cancel
</i18n>

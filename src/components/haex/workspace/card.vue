<template>
  <UCard
    ref="cardEl"
    class="cursor-pointer transition-all h-32 w-72 shrink-0 group duration-500 rounded-lg"
    :class="[
      workspace.id === currentWorkspace?.id
        ? 'ring-2 ring-secondary bg-secondary/10'
        : 'hover:ring-2 hover:ring-gray-300',
      isDragOver ? 'ring-4 ring-primary bg-primary/20 scale-105' : '',
    ]"
    @click="workspaceStore.slideToWorkspace(workspace.id)"
  >
    <template #header>
      <div class="flex justify-between">
        <h3 class="font-semibold text-gray-900 dark:text-white text-lg">
          {{ workspace.name }}
        </h3>

        <UButton
          v-if="workspaceStore.workspaces.length > 1"
          icon="mdi-close"
          variant="ghost"
          class="group-hover:opacity-100 opacity-0 transition-opacity duration-300"
          @click.stop="workspaceStore.closeWorkspaceAsync(workspace.id)"
        />
      </div>
    </template>

    <!-- Window Icons Preview -->
    <div
      v-if="workspaceWindows.length > 0"
      class="flex flex-wrap gap-2 items-center"
    >
      <!-- Show first 8 window icons -->
      <HaexIcon
        v-for="window in visibleWindows"
        :key="window.id"
        :name="window.icon || 'i-heroicons-window'"
        :tooltip="window.title"
        class="size-6 opacity-70"
      />

      <!-- Show remaining count badge if more than 8 windows -->
      <UBadge
        v-if="remainingCount > 0"
        color="neutral"
        variant="subtle"
        size="sm"
      >
        +{{ remainingCount }}
      </UBadge>
    </div>

    <!-- Empty state when no windows -->
    <div
      v-else
      class="text-sm text-gray-400 dark:text-gray-600 italic"
    >
      {{ t('noWindows') }}
    </div>
  </UCard>
</template>

<script setup lang="ts">
const props = defineProps<{ workspace: IWorkspace }>()

const { t } = useI18n()
const workspaceStore = useWorkspaceStore()
const windowManager = useWindowManagerStore()

const { currentWorkspace } = storeToRefs(workspaceStore)

// Get all windows for this workspace
const workspaceWindows = computed(() => {
  return windowManager.windows.filter(
    (window) => window.workspaceId === props.workspace.id,
  )
})

// Limit to 8 visible icons
const MAX_VISIBLE_ICONS = 8
const visibleWindows = computed(() => {
  return workspaceWindows.value.slice(0, MAX_VISIBLE_ICONS)
})

// Count remaining windows
const remainingCount = computed(() => {
  const remaining = workspaceWindows.value.length - MAX_VISIBLE_ICONS
  return remaining > 0 ? remaining : 0
})

const cardEl = useTemplateRef('cardEl')
const isDragOver = ref(false)

// Use mouse position to detect if over card
const { x: mouseX, y: mouseY } = useMouse()

// Check if mouse is over this card while dragging
watchEffect(() => {
  if (!windowManager.draggingWindowId || !cardEl.value?.$el) {
    isDragOver.value = false
    return
  }

  // Get card bounding box
  const rect = cardEl.value.$el.getBoundingClientRect()

  // Check if mouse is within card bounds
  const isOver =
    mouseX.value >= rect.left &&
    mouseX.value <= rect.right &&
    mouseY.value >= rect.top &&
    mouseY.value <= rect.bottom

  isDragOver.value = isOver
})

// Handle drop when drag ends - check BEFORE draggingWindowId is cleared
let wasOverThisCard = false

watchEffect(() => {
  if (isDragOver.value && windowManager.draggingWindowId) {
    wasOverThisCard = true
  }
})

watch(
  () => windowManager.draggingWindowId,
  (newValue, oldValue) => {
    // Drag ended (from something to null)
    if (oldValue && !newValue && wasOverThisCard) {
      console.log(
        '[WorkspaceCard] Drop detected! Moving window to workspace:',
        props.workspace.name,
      )
      const window = windowManager.windows.find((w) => w.id === oldValue)
      if (window) {
        window.workspaceId = props.workspace.id
        window.x = 0
        window.y = 0
        // Switch to the workspace after dropping
        //workspaceStore.slideToWorkspace(props.workspace.id)
      }
      wasOverThisCard = false
    } else if (!newValue) {
      // Drag ended but not over this card
      wasOverThisCard = false
    }
  },
)
</script>

<i18n lang="yaml">
de:
  noWindows: Keine Fenster geöffnet
en:
  noWindows: No windows open
</i18n>

<template>
  <UCard
    ref="cardEl"
    class="cursor-pointer transition-all h-32 w-72 shrink-0 group duration-500"
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
  </UCard>
</template>

<script setup lang="ts">
const props = defineProps<{ workspace: IWorkspace }>()

const workspaceStore = useWorkspaceStore()
const windowManager = useWindowManagerStore()

const { currentWorkspace } = storeToRefs(workspaceStore)

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

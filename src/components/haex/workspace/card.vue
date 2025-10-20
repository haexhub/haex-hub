<template>
  <UCard
    class="cursor-pointer transition-all h-32 w-72 shrink-0 group duration-500"
    :class="[
      workspace.position === currentWorkspaceIndex
        ? 'ring-2 ring-secondary bg-secondary/10'
        : 'hover:ring-2 hover:ring-gray-300',
    ]"
    @click="workspaceStore.slideToWorkspace(workspace.position)"
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
defineProps<{ workspace: IWorkspace }>()

const workspaceStore = useWorkspaceStore()

const { currentWorkspaceIndex } = storeToRefs(workspaceStore)
</script>

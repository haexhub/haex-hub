<template>
  <UiDrawer
    v-model:open="isOverviewMode"
    direction="left"
    :overlay="false"
    :modal="false"
    title="Workspaces"
    description="Workspaces"
  >
    <template #content>
      <div class="pl-8 pr-4 overflow-y-auto py-8">
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
          icon="i-heroicons-plus"
          :label="t('add')"
          @click="handleAddWorkspaceAsync"
        />
      </div>
    </template>
  </UiDrawer>
</template>

<script setup lang="ts">
const { t } = useI18n()

const workspaceStore = useWorkspaceStore()
const { workspaces, isOverviewMode } = storeToRefs(workspaceStore)

const handleAddWorkspaceAsync = async () => {
  const workspace = await workspaceStore.addWorkspaceAsync()
  nextTick(() => {
    workspaceStore.slideToWorkspace(workspace?.id)
  })
}
</script>

<i18n lang="yaml">
de:
  add: Workspace hinzufügen
en:
  add: Add Workspace
</i18n>

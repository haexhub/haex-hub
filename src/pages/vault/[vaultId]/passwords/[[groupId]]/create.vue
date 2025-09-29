<template>
  <div class="">
    <HaexPassGroup
      v-model="group"
      mode="create"
      @close="onClose"
      @submit="createAsync"
    />

    <HaexPassMenuBottom
      show-close-button
      show-save-button
      :has-changes
      @close="onClose"
      @save="createAsync"
    />

    <HaexPassDialogUnsavedChanges
      v-model:ignore-changes="ignoreChanges"
      v-model:open="showUnsavedChangesDialog"
      :has-changes
      @abort="showUnsavedChangesDialog = false"
      @confirm="onConfirmIgnoreChanges"
    />
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

definePageMeta({
  name: 'passwordGroupCreate',
})

const { currentGroupId } = storeToRefs(usePasswordGroupStore())
const group = ref<SelectHaexPasswordsGroups>({
  name: '',
  description: '',
  id: '',
  color: null,
  icon: null,
  order: null,
  parentId: currentGroupId.value || null,
  createdAt: null,
  updateAt: null,
  haex_tombstone: null,
})

const errors = ref({
  name: [],
  description: [],
})

const ignoreChanges = ref(false)

const onClose = () => {
  if (showUnsavedChangesDialog.value) return

  if (hasChanges.value && !ignoreChanges.value) {
    return (showUnsavedChangesDialog.value = true)
  }
  useRouter().back()
}

const { addGroupAsync } = usePasswordGroupStore()
const createAsync = async () => {
  try {
    if (errors.value.name.length || errors.value.description.length) return

    const newGroup = await addGroupAsync(group.value)

    if (!newGroup.id) {
      return
    }

    ignoreChanges.value = true
    await navigateTo(
      useLocalePath()({
        name: 'passwordGroupItems',
        params: {
          groupId: newGroup.id,
        },
        query: {
          ...useRoute().query,
        },
      }),
    )
  } catch (error) {
    console.log(error)
  }
}

const hasChanges = computed(() => {
  return !!(
    group.value.color ||
    group.value.description ||
    group.value.icon ||
    group.value.name
  )
})

const showUnsavedChangesDialog = ref(false)
const onConfirmIgnoreChanges = () => {
  showUnsavedChangesDialog.value = false
  ignoreChanges.value = true
  onClose()
}
</script>

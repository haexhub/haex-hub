<template>
  <div>
    <HaexPassGroup
      v-model="group"
      :read-only
      mode="edit"
      @close="onClose"
      @submit="onSaveAsync"
    />

    <HaexPassMenuBottom
      :show-edit-button="readOnly && !hasChanges"
      :show-readonly-button="!readOnly && !hasChanges"
      :show-save-button="hasChanges"
      :has-changes
      show-close-button
      show-delete-button
      @close="onClose()"
      @delete="showConfirmDeleteDialog = true"
      @edit="readOnly = false"
      @readonly="readOnly = true"
      @save="onSaveAsync"
    />

    <HaexPassDialogDeleteItem
      v-model:open="showConfirmDeleteDialog"
      :item-name="group.name"
      :final="inTrashGroup"
      @abort="showConfirmDeleteDialog = false"
      @confirm="onDeleteAsync"
    />

    <HaexPassDialogUnsavedChanges
      v-model:ignore-changes="ignoreChanges"
      v-model:open="showUnsavedChangesDialog"
      :has-changes="hasChanges"
      @abort="showUnsavedChangesDialog = false"
      @confirm="onConfirmIgnoreChanges"
    />
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

definePageMeta({
  name: 'passwordGroupEdit',
})

const { t } = useI18n()

const { inTrashGroup, currentGroupId } = storeToRefs(usePasswordGroupStore())

const group = ref<SelectHaexPasswordsGroups>({
  color: null,
  createdAt: null,
  description: null,
  icon: null,
  id: '',
  name: '',
  order: null,
  parentId: null,
  updateAt: null,
  haex_tombstone: null,
})

const original = ref<string>('')
const ignoreChanges = ref(false)

const { readGroupAsync } = usePasswordGroupStore()
watchImmediate(currentGroupId, async () => {
  if (!currentGroupId.value) return
  ignoreChanges.value = false
  try {
    const foundGroup = await readGroupAsync(currentGroupId.value)
    if (foundGroup) {
      original.value = JSON.parse(JSON.stringify(foundGroup))
      group.value = foundGroup
    }
  } catch (error) {
    console.error(error)
  }
})

const hasChanges = computed(() => {
  const current = JSON.stringify(group.value)
  const origin = JSON.stringify(original.value)
  console.log('hasChanges', current, origin)
  return !(current === origin)
})

const readOnly = ref(false)
const onClose = () => {
  if (showConfirmDeleteDialog.value || showUnsavedChangesDialog.value) return

  readOnly.value = true
  useRouter().back()
}

const { add } = useToast()

const { updateAsync, syncGroupItemsAsync, deleteGroupAsync } =
  usePasswordGroupStore()

const onSaveAsync = async () => {
  try {
    if (!group.value) return

    ignoreChanges.value = true
    await updateAsync(group.value)
    await syncGroupItemsAsync()
    add({ color: 'success', description: t('change.success') })
    onClose()
  } catch (error) {
    add({ color: 'error', description: t('change.error') })
    console.log(error)
  }
}

const showUnsavedChangesDialog = ref(false)
const onConfirmIgnoreChanges = () => {
  showUnsavedChangesDialog.value = false
  onClose()
}

const showConfirmDeleteDialog = ref(false)
const onDeleteAsync = async () => {
  try {
    const parentId = group.value.parentId
    await deleteGroupAsync(group.value.id, inTrashGroup.value)
    await syncGroupItemsAsync()
    showConfirmDeleteDialog.value = false
    ignoreChanges.value = true
    await navigateTo(
      useLocalePath()({
        name: 'passwordGroupItems',
        params: {
          ...useRouter().currentRoute.value.params,
          groupId: parentId,
        },
      }),
    )
  } catch (error) {
    console.error(error)
  }
}
</script>

<i18n lang="yaml">
de:
  title: Gruppe ändern
  abort: Abbrechen
  save: Speichern
  name:
    label: Name

  description:
    label: Beschreibung

  change:
    success: Änderung erfolgreich gespeichert
    error: Änderung konnte nicht gespeichert werden

en:
  title: Edit Group
  abort: Abort
  save: Save
  name:
    label: Name

  description:
    label: Description

  change:
    success: Change successfully saved
    error: Change could not be saved
</i18n>

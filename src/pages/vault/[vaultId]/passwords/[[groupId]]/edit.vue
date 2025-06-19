<template>
  <div>
    <HaexPassGroup
      :read_only
      @close="onClose"
      @submit="onSaveAsync"
      mode="edit"
      v-model="group"
    />

    <HaexPassMenuBottom
      :show-edit-button="read_only && !hasChanges"
      :show-readonly-button="!read_only && !hasChanges"
      :show-save-button="hasChanges"
      :has-changes
      @close="onClose()"
      @delete="showConfirmDeleteDialog = true"
      @edit="read_only = false"
      @readonly="read_only = true"
      @save="onSaveAsync"
      show-close-button
      show-delete-button
    >
    </HaexPassMenuBottom>

    <HaexPassDialogDeleteItem
      v-model:open="showConfirmDeleteDialog"
      @abort="showConfirmDeleteDialog = false"
      @confirm="onDeleteAsync"
      :item-name="group.name"
      :final="inTrashGroup"
    >
    </HaexPassDialogDeleteItem>

    <HaexPassDialogUnsavedChanges
      :has-changes="hasChanges"
      v-model:ignore-changes="ignoreChanges"
      @abort="showUnsavedChangesDialog = false"
      @confirm="onConfirmIgnoreChanges"
      v-model:open="showUnsavedChangesDialog"
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
  name: null,
  order: null,
  parentId: null,
  updateAt: null,
})

const original = ref<string>('')
const ignoreChanges = ref(false)

const { readGroupAsync } = usePasswordGroupStore()
watch(
  currentGroupId,
  async () => {
    if (!currentGroupId.value) return
    ignoreChanges.value = false
    try {
      const foundGroup = await readGroupAsync(currentGroupId.value)
      if (foundGroup) {
        original.value = JSON.stringify(foundGroup)
        group.value = foundGroup
      }
    } catch (error) {
      console.error(error)
    }
  },
  { immediate: true },
)
/* watch(
  currentGroup,
  (n, o) => {
    console.log('currentGroup', currentGroup.value, n, o)
    original.value = JSON.stringify(currentGroup.value)
    group.value = JSON.parse(original.value)
    ignoreChanges.value = false
  },
  { immediate: true },
) */

const read_only = ref(false)

const hasChanges = computed(
  () => JSON.stringify(group.value) !== original.value,
)

const onClose = () => {
  if (showConfirmDeleteDialog.value || showUnsavedChangesDialog.value) return

  read_only.value = true
  useRouter().back()
}

const { add } = useSnackbar()

const { updateAsync, syncGroupItemsAsync, deleteGroupAsync } =
  usePasswordGroupStore()

const onSaveAsync = async () => {
  try {
    if (!group.value) return

    ignoreChanges.value = true
    await updateAsync(group.value)
    await syncGroupItemsAsync(group.value.id)
    add({ type: 'success', text: t('change.success') })
    onClose()
  } catch (error) {
    add({ type: 'error', text: t('change.error') })
    console.log(error)
  }
}

const showConfirmDeleteDialog = ref(false)
const showUnsavedChangesDialog = ref(false)
const onConfirmIgnoreChanges = () => {
  showUnsavedChangesDialog.value = false
  onClose()
}

const onDeleteAsync = async () => {
  try {
    const parentId = group.value.parentId
    await deleteGroupAsync(group.value.id, inTrashGroup.value)
    await syncGroupItemsAsync(parentId)
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

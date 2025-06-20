<template>
  <div>
    <div class="flex flex-col">
      <p>
        {{ item.originalDetails }}
      </p>
      {{ item.details }}
    </div>
    <HaexPassItem
      :history="item.history"
      :read_only
      @close="onClose()"
      @submit="onUpdateAsync"
      v-model:details="item.details"
      v-model:key-values-add="item.keyValuesAdd"
      v-model:key-values-delete="item.keyValuesDelete"
      v-model:key-values="item.keyValues"
    />

    <HaexPassMenuBottom
      :has-changes
      :show-edit-button="read_only && !hasChanges"
      :show-readonly-button="!read_only && !hasChanges"
      :show-save-button="!read_only && hasChanges"
      @close="onClose"
      @delete="showConfirmDeleteDialog = true"
      @edit="read_only = false"
      @readonly="read_only = true"
      @save="onUpdateAsync"
      show-close-button
      show-delete-button
    >
    </HaexPassMenuBottom>

    <HaexPassDialogDeleteItem
      v-model:open="showConfirmDeleteDialog"
      @abort="showConfirmDeleteDialog = false"
      @confirm="deleteItemAsync"
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
import { usePasswordGroup } from '~/components/haex/pass/group/composables'
import type {
  SelectHaexPasswordsItemDetails,
  SelectHaexPasswordsItemHistory,
  SelectHaexPasswordsItemKeyValues,
} from '~~/src-tauri/database/schemas/vault'

definePageMeta({
  name: 'passwordItemEdit',
})

defineProps({
  icon: String,
  title: String,
  withCopyButton: Boolean,
})

const read_only = ref(true)
const showConfirmDeleteDialog = ref(false)
const { t } = useI18n()

const item = reactive<{
  details: SelectHaexPasswordsItemDetails
  history: SelectHaexPasswordsItemHistory[]
  keyValues: SelectHaexPasswordsItemKeyValues[]
  keyValuesAdd: SelectHaexPasswordsItemKeyValues[]
  keyValuesDelete: SelectHaexPasswordsItemKeyValues[]
  originalDetails: SelectHaexPasswordsItemDetails | null
  originalKeyValues: SelectHaexPasswordsItemKeyValues[] | null
}>({
  details: {
    id: '',
    createdAt: null,
    icon: null,
    note: null,
    password: null,
    tags: null,
    title: null,
    updateAt: null,
    url: null,
    username: null,
  },
  keyValues: [],
  history: [],
  keyValuesAdd: [],
  keyValuesDelete: [],
  originalDetails: {
    id: '',
    createdAt: null,
    icon: null,
    note: null,
    password: null,
    tags: null,
    title: null,
    updateAt: null,
    url: null,
    username: null,
  },
  originalKeyValues: null,
})

const { currentItem } = storeToRefs(usePasswordItemStore())

watch(
  currentItem,
  () => {
    if (!currentItem.value) return
    item.details = JSON.parse(JSON.stringify(currentItem.value?.details))
    item.keyValues = JSON.parse(JSON.stringify(currentItem.value?.keyValues))
    item.history = JSON.parse(JSON.stringify(currentItem.value?.history))
    item.keyValuesAdd = []
    item.keyValuesDelete = []
    item.originalDetails = JSON.parse(
      JSON.stringify(currentItem.value?.details),
    )
    item.originalKeyValues = JSON.parse(
      JSON.stringify(currentItem.value?.keyValues),
    )
  },
  { immediate: true },
)

const { add } = useSnackbar()
const { deleteAsync, updateAsync } = usePasswordItemStore()
const { syncGroupItemsAsync } = usePasswordGroupStore()
const { currentGroupId, inTrashGroup } = storeToRefs(usePasswordGroupStore())

const ignoreChanges = ref(false)
const onUpdateAsync = async () => {
  try {
    const newId = await updateAsync({
      details: item.details,
      groupId: currentGroupId.value || null,
      keyValues: item.keyValues,
      keyValuesAdd: item.keyValuesAdd,
      keyValuesDelete: item.keyValuesDelete,
    })
    if (newId) add({ type: 'success', text: t('success.update') })
    syncGroupItemsAsync()
    ignoreChanges.value = true
    onClose()
  } catch (error) {
    add({ type: 'error', text: t('error.update') })
  }
}

const onClose = () => {
  if (showConfirmDeleteDialog.value || showUnsavedChangesDialog.value) return

  if (hasChanges.value && !ignoreChanges.value)
    return (showUnsavedChangesDialog.value = true)

  read_only.value = true
  useRouter().back()
}

const deleteItemAsync = async () => {
  try {
    await deleteAsync(item.details.id, inTrashGroup.value)
    showConfirmDeleteDialog.value = false
    add({ type: 'success', text: t('success.delete') })
    await syncGroupItemsAsync()
    onClose()
  } catch (errro) {
    add({
      type: 'error',
      text: t('error.delete'),
    })
  }
}

const { areItemsEqual } = usePasswordGroup()
const hasChanges = computed(
  () =>
    !!(
      !areItemsEqual(item.originalDetails, item.details) ||
      !areItemsEqual(item.originalKeyValues, item.keyValues) ||
      item.keyValuesAdd.length ||
      item.keyValuesDelete.length
    ),
)

const showUnsavedChangesDialog = ref(false)
const onConfirmIgnoreChanges = () => {
  showUnsavedChangesDialog.value = false
  ignoreChanges.value = true
  onClose()
}
</script>

<i18n lang="yaml">
de:
  success:
    update: Eintrag erfolgreich aktualisiert
    delete: Eintrag wurde gelöscht
  error:
    update: Eintrag konnte nicht aktualisiert werden
    delete: Eintrag konnte nicht gelöscht werden
  tab:
    details: Details
    keyValue: Extra
    history: Verlauf

en:
  success:
    update: Entry successfully updated
    delete: Entry successfully removed
  error:
    update: Entry could not be updated
    delete: Entry could not be deleted
  tab:
    details: Details
    keyValue: Extra
    history: History
</i18n>

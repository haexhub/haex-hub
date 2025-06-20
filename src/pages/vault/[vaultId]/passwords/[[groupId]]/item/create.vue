<template>
  <div>
    {{ currentGroup?.id }} {{ currentGroupId }}
    <HaexPassItem
      :default-icon="currentGroup?.icon"
      :history="item.history"
      @close="onClose"
      @submit="onCreateAsync"
      v-model:details="item.details"
      v-model:key-values-add="item.keyValuesAdd"
    />

    <HaexPassMenuBottom
      :has-changes
      @close="onClose"
      @save="onCreateAsync"
      show-close-button
      show-save-button
    >
    </HaexPassMenuBottom>

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
import type {
  SelectHaexPasswordsItemDetails,
  SelectHaexPasswordsItemHistory,
  SelectHaexPasswordsItemKeyValues,
} from '~~/src-tauri/database/schemas/vault'

definePageMeta({
  name: 'passwordItemCreate',
})

defineProps({
  icon: String,
  title: String,
  withCopyButton: Boolean,
})

const { t } = useI18n()

const item = reactive<{
  details: SelectHaexPasswordsItemDetails
  history: SelectHaexPasswordsItemHistory[]
  keyValuesAdd: SelectHaexPasswordsItemKeyValues[]
  originalDetails: SelectHaexPasswordsItemDetails
  originalKeyValuesAdd: []
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
  history: [],
  keyValuesAdd: [],
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
  originalKeyValuesAdd: [],
})

const { add } = useSnackbar()
const { currentGroup, currentGroupId } = storeToRefs(usePasswordGroupStore())
const { syncGroupItemsAsync } = usePasswordGroupStore()
const { addAsync } = usePasswordItemStore()

const onCreateAsync = async () => {
  try {
    const newId = await addAsync(
      item.details,
      item.keyValuesAdd,
      currentGroup.value,
    )

    if (newId) {
      ignoreChanges.value = true
      add({ type: 'success', text: t('success') })
      await syncGroupItemsAsync()
      onClose()
    }
  } catch (error) {
    add({ type: 'error', text: t('error') })
  }
}

const ignoreChanges = ref(false)

const onClose = () => {
  if (showUnsavedChangesDialog.value) return

  if (hasChanges.value && !ignoreChanges.value)
    return (showUnsavedChangesDialog.value = true)

  useRouter().back()
}

const { areItemsEqual } = usePasswordGroup()
const hasChanges = computed(
  () =>
    !!(
      !areItemsEqual(item.originalDetails, item.details) ||
      item.keyValuesAdd.length
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
  create: Anlegen
  abort: Abbrechen
  success: Eintrag erfolgreich erstellt
  error: Eintrag konnte nicht erstellt werden
  tab:
    details: Details
    keyValue: Extra
    history: Verlauf
en:
  create: Create
  abort: Abort
  success: Entry successfully created
  error: Entry could not be created
  tab:
    details: Details
    keyValue: Extra
    history: History
</i18n>

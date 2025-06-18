<template>
  <div>
    <HaexPassItem
      :default-icon="currentGroup?.icon"
      :history="item.history"
      @close="onClose"
      @submit="onCreateAsync"
      v-model:details="item.details"
      v-model:key-values-add="item.keyValuesAdd"
    />

    <div
      class="fixed bottom-4 flex justify-between transition-all pointer-events-none right-0 sm:items-center items-end"
      :class="[isVisible ? 'left-15 ' : 'left-0']"
    >
      <div class="flex items-center justify-center flex-1">
        <UiTooltip :tooltip="t('abort')">
          <UiButton
            class="btn-error btn-square"
            @click="onClose"
          >
            <Icon name="mdi:close" />
          </UiButton>
        </UiTooltip>
      </div>
      <UiTooltip :tooltip="t('create')">
        <UiButton
          class="btn-xl btn-square btn-primary"
          @click="onCreateAsync"
        >
          <Icon
            name="mdi:content-save-outline"
            class="size-11 shrink-0"
          />
        </UiButton>
      </UiTooltip>
      <div class="flex items-center justify-center flex-1"></div>
    </div>
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

const { isVisible } = storeToRefs(useSidebarStore())

const { t } = useI18n()

const item = reactive<{
  details: SelectHaexPasswordsItemDetails
  history: SelectHaexPasswordsItemHistory[]
  keyValuesAdd: SelectHaexPasswordsItemKeyValues[]
  keyValuesDelete: SelectHaexPasswordsItemKeyValues[]
  originalDetails: string | null
  originalKeyValues: string | null
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
  keyValuesDelete: [],
  originalDetails: null,
  originalKeyValues: null,
})

const { add } = useSnackbar()
const { currentGroup } = storeToRefs(usePasswordGroupStore())
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
      add({ type: 'success', text: t('success') })
      syncGroupItemsAsync(currentGroup.value?.id)
      onClose()
    }
  } catch (error) {
    add({ type: 'error', text: t('error') })
  }
}

const onClose = () => useRouter().back()
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

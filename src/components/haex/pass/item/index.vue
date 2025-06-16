<template>
  <UiCard
    body-class="rounded overflow-auto px-0 h-full"
    @close="onClose"
  >
    <div class="">
      <nav
        aria-label="Tabs Password Item"
        aria-orientation="horizontal"
        class="tabs tabs-bordered w-full transition-all duration-700 sticky top-0 z-10"
        role="tablist"
      >
        <button
          :id="id.details"
          aria-controls="vaultDetailsId"
          aria-selected="true"
          class="tab active-tab:tab-active active w-full"
          data-tab="#vaultDetailsId"
          role="tab"
          type="button"
        >
          <Icon
            name="material-symbols:key-outline"
            class="me-2"
          />
          <span class="hidden sm:block">
            {{ t('tab.details') }}
          </span>
        </button>
        <button
          :id="id.keyValue"
          aria-controls="tabs-basic-2"
          aria-selected="false"
          class="tab active-tab:tab-active w-full"
          data-tab="#tabs-basic-2"
          role="tab"
          type="button"
        >
          <Icon
            name="fluent:group-list-20-filled"
            class="me-2"
          />
          <span class="hidden sm:block">
            {{ t('tab.keyValue') }}
          </span>
        </button>
        <button
          :id="id.history"
          aria-controls="tabs-basic-3"
          aria-selected="false"
          class="tab active-tab:tab-active w-full"
          data-tab="#tabs-basic-3"
          role="tab"
          type="button"
        >
          <Icon
            name="material-symbols:history"
            class="me-2"
          />
          <span class="hidden sm:block">
            {{ t('tab.history') }}
          </span>
        </button>
      </nav>

      <div class="h-full pb-8">
        <div
          id="vaultDetailsId"
          role="tabpanel"
          class="h-full"
          :aria-labelledby="id.details"
        >
          <HaexPassItemDetails
            v-if="details"
            v-model="details"
            with-copy-button
            :read_only
            :defaultIcon
            v-model:prevent-close="preventClose"
          />
        </div>

        <div
          id="tabs-basic-2"
          class="hidden"
          role="tabpanel"
          :aria-labelledby="id.keyValue"
        >
          <HaexPassItemKeyValue
            v-if="keyValues"
            v-model="keyValues"
            v-model:items-to-add="keyValuesAdd"
            v-model:items-to-delete="keyValuesDelete"
            :read_only
            :item-id="details!.id"
          />
        </div>

        <div
          id="tabs-basic-3"
          class="hidden h-full"
          role="tabpanel"
          :aria-labelledby="id.history"
        >
          <!-- <HaexPassItemHistory v-model="itemHistory" /> -->
        </div>
      </div>
    </div>
  </UiCard>
</template>

<script setup lang="ts">
import type {
  SelectHaexPasswordsItemDetails,
  SelectHaexPasswordsItemHistory,
  SelectHaexPasswordsItemKeyValues,
} from '~~/src-tauri/database/schemas/vault'

defineProps<{
  defaultIcon?: string | null
  history: SelectHaexPasswordsItemHistory[]
}>()

const emit = defineEmits<{
  close: [void]
  addKeyValue: [void]
  removeKeyValue: [string]
}>()

const read_only = defineModel<boolean>('read_only', { default: false })

const details = defineModel<SelectHaexPasswordsItemDetails | null>('details', {
  required: true,
})

const keyValues = defineModel<SelectHaexPasswordsItemKeyValues[]>('keyValues', {
  default: [],
})

const keyValuesAdd = defineModel<SelectHaexPasswordsItemKeyValues[]>(
  'keyValuesAdd',
  { default: [] },
)
const keyValuesDelete = defineModel<SelectHaexPasswordsItemKeyValues[]>(
  'keyValuesDelete',
  { default: [] },
)

const { t } = useI18n()

const id = reactive({
  details: useId(),
  keyValue: useId(),
  history: useId(),
  content: {},
})

const preventClose = ref(false)

const onClose = () => {
  if (preventClose.value) return

  emit('close')
}
</script>

<i18n lang="json">
{
  "de": {
    "create": "Anlegen",
    "abort": "Abbrechen",
    "tab": {
      "details": "Details",
      "keyValue": "Extra",
      "history": "Verlauf"
    }
  },
  "en": {
    "create": "Create",
    "abort": "Abort",
    "tab": {
      "details": "Details",
      "keyValue": "Extra",
      "history": "History"
    }
  }
}
</i18n>

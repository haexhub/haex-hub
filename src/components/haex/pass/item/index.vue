<template>
  <div class="p-1">
    <UCard
      class="rounded overflow-auto p-0 h-full"
      @close="onClose"
    >
      <div class="">
        <UTabs
          :items="tabs"
          variant="link"
          :ui="{ trigger: 'grow' }"
          class="gap-4 w-full"
        >
          <template #details>
            <HaexPassItemDetails
              v-if="details"
              v-model="details"
              with-copy-button
              :read-only
              :defaultIcon
              v-model:prevent-close="preventClose"
              @submit="$emit('submit')"
            />
          </template>

          <template #keyValue>
            <HaexPassItemKeyValue
              v-if="keyValues"
              v-model="keyValues"
              v-model:items-to-add="keyValuesAdd"
              v-model:items-to-delete="keyValuesDelete"
              :read-only
              :item-id="details!.id"
            />
          </template>
        </UTabs>

        <!-- <div class="h-full pb-8">
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
              @submit="$emit('submit')"
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
            <HaexPassItemHistory />
          </div>
        </div> -->
      </div>
    </UCard>
  </div>
</template>

<script setup lang="ts">
import type { TabsItem } from '@nuxt/ui'
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
  close: []
  addKeyValue: []
  removeKeyValue: [string]
  submit: []
}>()

const readOnly = defineModel<boolean>('readOnly', { default: false })

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

/* const id = reactive({
  details: useId(),
  keyValue: useId(),
  history: useId(),
  content: {},
}) */

const preventClose = ref(false)

const onClose = () => {
  if (preventClose.value) return

  emit('close')
}

const tabs = ref<TabsItem[]>([
  {
    label: t('tab.details'),
    icon: 'material-symbols:key-outline',
    slot: 'details' as const,
  },
  {
    label: t('tab.keyValue'),
    icon: 'fluent:group-list-20-filled',
    slot: 'keyValue' as const,
  },
  {
    label: t('tab.history'),
    icon: 'material-symbols:history',
    slot: 'history' as const,
  },
])
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

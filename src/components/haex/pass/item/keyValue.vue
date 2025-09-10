<template>
  <div class="p-4">
    <div class="flex flex-wrap gap-2">
      <UiList
        v-if="items.length || itemsToAdd.length"
        class="flex-1"
      >
        <li
          v-for="item in [...items, ...itemsToAdd]"
          :key="item.id"
          :class="{ 'bg-primary/20': currentSelected === item }"
          class="flex gap-2 hover:bg-primary/20 px-4 items-center"
          @click="currentSelected = item"
        >
          <button class="flex items-center no-underline w-full py-2">
            <input
              v-model="item.key"
              :readonly="currentSelected !== item || readOnly"
              class="flex-1 cursor-pointer"
            />
          </button>

          <UiButton
            v-if="!readOnly"
            :class="[currentSelected === item ? 'visible' : 'invisible']"
            variant="outline"
            color="error"
            icon="mdi:trash-outline"
            @click="deleteItem(item.id)"
          />
        </li>
      </UiList>

      <UTextarea
        v-if="items.length || itemsToAdd.length"
        :readOnly="readOnly || !currentSelected"
        class="flex-1 min-w-52 border-base-content/25"
        v-model="currentValue"
        with-copy-button
      />
    </div>
    <div
      v-show="!readOnly"
      class="flex py-4 gap-2 justify-center items-end flex-wrap"
    >
      <UiButton
        @click="addItem"
        class="btn-primary btn-outline flex-1-1 min-w-40"
        icon="mdi:plus"
      >
        <!-- <Icon name="mdi:plus" />
        <p class="hidden sm:inline-block">{{ t('add') }}</p> -->
      </UiButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsItemKeyValues } from '~~/src-tauri/database/schemas/vault'

const { itemId } = defineProps<{ readOnly?: boolean; itemId: string }>()

const items = defineModel<SelectHaexPasswordsItemKeyValues[]>({ default: [] })

const itemsToDelete = defineModel<SelectHaexPasswordsItemKeyValues[]>(
  'itemsToDelete',
  { default: [] },
)
const itemsToAdd = defineModel<SelectHaexPasswordsItemKeyValues[]>(
  'itemsToAdd',
  { default: [] },
)

defineEmits<{ add: []; remove: [string] }>()

//const { t } = useI18n()

const currentSelected = ref<SelectHaexPasswordsItemKeyValues | undefined>(
  items.value?.at(0),
)

watch(
  () => itemId,
  () => (currentSelected.value = items.value?.at(0)),
)
//const currentValue = computed(() => currentSelected.value?.value || '')
const currentValue = computed({
  get: () => currentSelected.value?.value || '',
  set(newValue: string) {
    if (currentSelected.value) currentSelected.value.value = newValue
  },
})

const addItem = () => {
  itemsToAdd.value?.push({
    id: crypto.randomUUID(),
    itemId,
    key: '',
    value: '',
    updateAt: null,
    haex_tombstone: null,
  })
}

const deleteItem = (id: string) => {
  const item = items.value.find((item) => item.id === id)
  if (item) {
    itemsToDelete.value?.push(item)
    items.value = items.value.filter((item) => item.id !== id)
  }

  itemsToAdd.value = itemsToAdd.value?.filter((item) => item.id !== id) ?? []
}
</script>

<i18n lang="yaml">
de:
  add: Hinzufügen
  key: Schlüssel
  value: Wert

en:
  add: Add
  key: Key
  value: Value
</i18n>

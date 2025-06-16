<template>
  <div class="h-full overflow-scroll flex">
    {{ _history }}
    <UiList v-show="_history.length">
      <!-- <UiListButton v-for="item in _history">
        <div
          class="flex items-start bg-slate-100 gap-x-2 w-full h-20 overflow-clip"
        >
          <div class="flex flex-col justify-between h-full py-2">
            <h6 class="text-sm whitespace-nowrap bg-orange-200">
              vorheriger {{ item.changedProperty }}
            </h6>
            <UiInput
              :model-value="item.oldValue"
              with-copy-button
            />
          </div>

          <div class="sm:flex flex-col justify-between h-full py-2 hidden">
            <h6 class="text-sm">neuer Wert</h6>
            <UiInput
              :model-value="item.newValue"
              with-copy-button
            />
          </div>

          <div class="flex flex-col justify-between h-full py-2">
            <h6 class="text-sm md:text-base bg-orange-200">geändert_am</h6>
            <span class="bg-red-100 py-1 md:py-2">
              {{ item.createdAt }}
            </span>
          </div>
        </div>
      </UiListButton> 
    -->
    </UiList>

    <div
      v-show="!_history.length"
      class="content-center w-full text-center"
    >
      {{ t('noHistory') }}
    </div>
  </div>
  <!-- <UiTable
    v-if="history?.length"
    :headers
    :items="_history"
    autofocus
  >
    <template #column-oldValue="{ item }: { item: string }">
      <UiInput
        :model-value="item"
        with-copy-button
        class="min-w-24"
      />
    </template>
  </UiTable> -->
</template>

<script setup lang="ts">
import type {
  SelectHaexPasswordsGroupItems,
  SelectHaexPasswordsItemDetails,
  SelectHaexPasswordsItemHistory,
} from '~~/src-tauri/database/schemas/vault'

const history = defineModel<SelectHaexPasswordsItemHistory[]>()

const _history = computed(
  () =>
    history.value?.map((change) => ({
      changedProperty: t(change.changedProperty!),
      createdAt: new Date(change.createdAt!).toLocaleDateString(),
      newValue: change.newValue,
      oldValue: change.oldValue,
    })) ?? [],
)

const { t } = useI18n()
interface ITableHeader {
  label?: string
  'item-value': string
}
const headers: ITableHeader[] = [
  { 'item-value': 'changedProperty', label: t('changedProperty') },
  { 'item-value': 'oldValue', label: t('oldValue') },
  { 'item-value': 'newValue', label: t('newValue') },
  { 'item-value': 'createdAt', label: t('createdAt') },
]
</script>

<i18n lang="json">
{
  "de": {
    "noHistory": "Eintrag wurde bisher nicht geändert",
    "changedProperty": "Änderung",
    "createdAt": "geändert am",
    "newValue": "neuer Wert",
    "oldValue": "alter Wert",
    "password": "Passwort",
    "title": "Titel",
    "url": "Url",
    "username": "Nutzername"
  },
  "en": {
    "noHistory": "No changes so far",
    "changedProperty": "Changes",
    "createdAt": "changed at",
    "newValue": "new Value",
    "oldValue": "old Value",
    "password": "Password",
    "title": "Title",
    "url": "Url",
    "username": "Username"
  }
}
</i18n>

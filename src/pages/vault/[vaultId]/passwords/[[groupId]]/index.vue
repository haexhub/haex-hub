<template>
  <div class="flex flex-1">
    <!-- <div class="h-screen bg-accented">aaa</div> -->
    <div class="flex flex-col flex-1">
      <HaexPassGroupBreadcrumbs
        v-show="breadCrumbs.length"
        :items="breadCrumbs"
        class="px-2 sticky -top-2 z-10"
      />
      <!-- <div class="flex-1 py-1 flex"> -->
      <HaexPassMobileMenu
        ref="listRef"
        v-model:selected-items="selectedItems"
        :menu-items="groupItems"
      />
      <!--  </div> -->

      <div
        class="fixed bottom-16 flex justify-between transition-all w-full sm:items-center items-end px-8 z-40"
      >
        <div class="w-full" />

        <UDropdownMenu
          v-model:open="open"
          :items="menu"
        >
          <UButton
            icon="mdi:plus"
            :ui="{
              base: 'rotate-45 z-40',
              leadingIcon: [open ? 'rotate-0' : 'rotate-45', 'transition-all'],
            }"
            size="xl"
          />
        </UDropdownMenu>

        <div
          class="flex flex-col sm:flex-row gap-4 w-full justify-end items-end"
        >
          <UiButton
            v-show="selectedItems.size === 1"
            color="secondary"
            icon="mdi:pencil"
            :tooltip="t('edit')"
            @click="onEditAsync"
          />

          <UiButton
            v-show="selectedItems.size"
            color="secondary"
            :tooltip="t('cut')"
            icon="mdi:scissors"
            @click="onCut"
          />

          <UiButton
            v-show="selectedGroupItems?.length"
            color="secondary"
            icon="proicons:clipboard-paste"
            :tooltip="t('paste')"
            @click="onPasteAsync"
          />

          <UiButton
            v-show="selectedItems.size"
            color="secondary"
            icon="mdi:trash-outline"
            :tooltip="t('delete')"
            @click="onDeleteAsync"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { IPasswordMenuItem } from '~/components/haex/pass/mobile/menu/types'
//import { useMagicKeys, whenever } from '@vueuse/core'
import Fuse from 'fuse.js'

definePageMeta({
  name: 'passwordGroupItems',
})

const open = ref(false)

const { t } = useI18n()

const { add } = useToast()

const selectedItems = ref<Set<IPasswordMenuItem>>(new Set())
const { menu } = storeToRefs(usePasswordsActionMenuStore())

const { syncItemsAsync } = usePasswordItemStore()
const { syncGroupItemsAsync } = usePasswordGroupStore()
onMounted(async () => {
  try {
    await Promise.allSettled([syncItemsAsync(), syncGroupItemsAsync()])
  } catch (error) {
    console.error(error)
  }
})

const {
  breadCrumbs,
  currentGroupId,
  inTrashGroup,
  selectedGroupItems,
  groups,
} = storeToRefs(usePasswordGroupStore())

const { items } = storeToRefs(usePasswordItemStore())
const { search } = storeToRefs(useSearchStore())

const groupItems = computed<IPasswordMenuItem[]>(() => {
  const menuItems: IPasswordMenuItem[] = []
  const filteredGroups = search.value
    ? new Fuse(groups.value, {
        keys: ['name', 'description'],
        findAllMatches: true,
      })
        .search(search.value)
        .map((match) => match.item)
    : groups.value.filter((group) => group.parentId == currentGroupId.value)

  const filteredItems = search.value
    ? new Fuse(items.value, {
        keys: ['title', 'note', 'password', 'tags', 'url', 'username'],
      })
        .search(search.value)
        .map((match) => match.item)
    : items.value.filter(
        (item) =>
          item.haex_passwords_group_items.groupId == currentGroupId.value,
      )

  menuItems.push(
    ...filteredGroups.map<IPasswordMenuItem>((group) => ({
      color: group.color,
      icon: group.icon,
      id: group.id,
      name: group.name,
      type: 'group',
    })),
  )

  menuItems.push(
    ...filteredItems.map<IPasswordMenuItem>((item) => ({
      icon: item.haex_passwords_item_details.icon,
      id: item.haex_passwords_item_details.id,
      name: item.haex_passwords_item_details.title,
      type: 'item',
    })),
  )

  return menuItems
})

const onEditAsync = async () => {
  const item = selectedItems.value.values().next().value

  if (item?.type === 'group')
    await navigateTo(
      useLocalePath()({
        name: 'passwordGroupEdit',
        params: { groupId: item.id },
      }),
    )
  else if (item?.type === 'item') {
    await navigateTo(
      useLocalePath()({
        name: 'passwordItemEdit',
        params: { itemId: item.id },
      }),
    )
  }
}
onKeyStroke('e', async (e) => {
  if (e.ctrlKey) {
    await onEditAsync()
  }
})

const onCut = () => {
  selectedGroupItems.value = [...selectedItems.value]
  selectedItems.value.clear()
}
onKeyStroke('x', (event) => {
  if (event.ctrlKey && selectedItems.value.size) {
    event.preventDefault()
    onCut()
  }
})

const { insertGroupItemsAsync } = usePasswordGroupStore()

const onPasteAsync = async () => {
  if (!selectedGroupItems.value?.length) return

  try {
    await insertGroupItemsAsync(
      [...selectedGroupItems.value],
      currentGroupId.value,
    )
    await syncGroupItemsAsync()
    selectedGroupItems.value = []
    selectedItems.value.clear()
  } catch (error) {
    console.error(error)
    selectedGroupItems.value = []
    add({ color: 'error', description: t('error.paste') })
  }
}
onKeyStroke('v', async (event) => {
  if (event.ctrlKey) {
    await onPasteAsync()
  }
})

/* const { escape } = useMagicKeys()
whenever(escape, () => {
  selectedItems.value.clear()
}) */

onKeyStroke('escape', () => selectedItems.value.clear())

onKeyStroke('a', (event) => {
  if (event.ctrlKey) {
    event.preventDefault()
    event.stopImmediatePropagation()
    selectedItems.value = new Set(groupItems.value)
  }
})

const { deleteAsync } = usePasswordItemStore()
const { deleteGroupAsync } = usePasswordGroupStore()

const onDeleteAsync = async () => {
  for (const item of selectedItems.value) {
    if (item.type === 'group') {
      await deleteGroupAsync(item.id, inTrashGroup.value)
    }
    if (item.type === 'item') {
      await deleteAsync(item.id, inTrashGroup.value)
    }
  }
  selectedItems.value.clear()
  await syncGroupItemsAsync()
}
/* const keys = useMagicKeys()
whenever(keys, async () => {
  await onDeleteAsync()
}) */
onKeyStroke('delete', () => onDeleteAsync())

const listRef = useTemplateRef<HTMLElement>('listRef')
onClickOutside(listRef, () => setTimeout(() => selectedItems.value.clear(), 50))
</script>

<i18n lang="yaml">
de:
  cut: Ausschneiden
  paste: Einfügen
  delete: Löschen
  edit: Bearbeiten
  wtf: 'wtf'
en:
  cut: Cut
  paste: Paste
  delete: Delete
  edit: Edit
</i18n>

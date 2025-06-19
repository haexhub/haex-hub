<template>
  <div class="h-full">
    <div class="min-h-full flex flex-col">
      <HaexPassGroupBreadcrumbs
        :items="breadCrumbs"
        class="px-2 sticky -top-2 z-10 bg-base-200"
        v-show="breadCrumbs.length"
      />
      <div class="flex-1 overflow-auto py-1">
        <HaexPassMobileMenu
          :menu-items="groupItems"
          ref="listRef"
          v-model:selected-items="selectedItems"
        />
      </div>

      <div
        class="fixed bottom-4 flex justify-between transition-all pointer-events-none right-0 sm:items-center items-end px-8"
        :class="[isVisible ? 'left-15 ' : 'left-0']"
      >
        <div class="w-full"></div>
        <UiButtonAction
          v-if="!inTrashGroup"
          :menu
        />

        <div
          class="flex flex-col sm:flex-row gap-4 w-full justify-end items-end"
        >
          <UiButton
            v-show="selectedItems.size === 1"
            class="btn-square btn-accent"
            @click="onEditAsync"
            :tooltip="t('edit')"
          >
            <Icon name="mdi:pencil" />
          </UiButton>

          <UiButton
            class="btn-square btn-accent"
            v-show="selectedItems.size"
            @click="onCut"
            :tooltip="t('cut')"
          >
            <Icon name="mdi:scissors" />
          </UiButton>

          <UiButton
            class="btn-square btn-accent"
            v-show="selectedGroupItems?.length"
            @click="onPasteAsync"
            :tooltip="t('paste')"
          >
            <Icon name="proicons:clipboard-paste" />
          </UiButton>

          <UiButton
            v-show="selectedItems.size"
            class="btn-square btn-accent"
            @click="onDeleteAsync"
            :tooltip="t('delete')"
          >
            <Icon name="mdi:trash" />
          </UiButton>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { IPasswordMenuItem } from '~/components/haex/pass/mobile/menu/types'
import { useMagicKeys } from '@vueuse/core'

definePageMeta({
  name: 'passwordGroupItems',
})

const { t } = useI18n()
const { add } = useSnackbar()

const selectedItems = ref<Set<IPasswordMenuItem>>(new Set())
const { menu } = storeToRefs(usePasswordsActionMenuStore())

const { syncItemsAsync } = usePasswordItemStore()
const { syncGroupItemsAsync } = usePasswordGroupStore()
onMounted(async () => {
  try {
    await Promise.allSettled([syncItemsAsync(), syncGroupItemsAsync()])
  } catch (error) {}
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

  menuItems.push(
    ...groups.value
      .filter((group) => {
        if (!search.value) return group.parentId == currentGroupId.value

        return (
          group.name?.includes(search.value) ||
          group.description?.includes(search.value)
        )
      })
      .map<IPasswordMenuItem>((group) => ({
        color: group.color,
        icon: group.icon,
        id: group.id,
        name: group.name,
        type: 'group',
      })),
  )

  menuItems.push(
    ...items.value
      .filter((item) => {
        if (!search.value)
          return item.haex_passwords_group_items.groupId == currentGroupId.value

        return (
          item.haex_passwords_item_details.title?.includes(search.value) ||
          item.haex_passwords_item_details.note?.includes(search.value) ||
          item.haex_passwords_item_details.password?.includes(search.value) ||
          item.haex_passwords_item_details.tags?.includes(search.value) ||
          item.haex_passwords_item_details.url?.includes(search.value) ||
          item.haex_passwords_item_details.username?.includes(search.value)
        )
      })
      .map<IPasswordMenuItem>((item) => ({
        icon: item.haex_passwords_item_details.icon,
        id: item.haex_passwords_item_details.id,
        name: item.haex_passwords_item_details.title,
        type: 'item',
      })),
  )

  return menuItems
})

const { isVisible } = storeToRefs(useSidebarStore())

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
    await syncGroupItemsAsync(currentGroupId.value)
    selectedGroupItems.value = []
    selectedItems.value.clear()
  } catch (error) {
    console.error(error)
    selectedGroupItems.value = []
    add({ type: 'error', text: t('error.paste') })
  }
}
onKeyStroke('v', async (event) => {
  if (event.ctrlKey) {
    event.preventDefault()
    await onPasteAsync()
  }
})

const { escape } = useMagicKeys()
watch(escape, () => {
  selectedItems.value.clear()
})

onKeyStroke('a', (event) => {
  if (event.ctrlKey) {
    event.preventDefault()
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
  await syncGroupItemsAsync(currentGroupId.value)
}
const keys = useMagicKeys()
watch(keys.delete, async () => {
  await onDeleteAsync()
})

const listRef = useTemplateRef<HTMLElement>('listRef')
onClickOutside(listRef, () => setTimeout(() => selectedItems.value.clear(), 50))
</script>

<i18n lang="yaml">
de:
  cut: Ausschneiden
  paste: Einfügen
  delete: Löschen
  edit: Bearbeiten
en:
  cut: Cut
  paste: Paste
  delete: Delete
  edit: Edit
</i18n>

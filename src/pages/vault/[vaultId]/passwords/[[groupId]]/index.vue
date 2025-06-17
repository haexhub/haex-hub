<template>
  <div class="relative h-full">
    <div class="h-full">
      <div class="h-full overflow-auto flex flex-col">
        <HaexPassGroupBreadcrumbs
          :items="breadCrumbs"
          class="px-2"
          v-show="breadCrumbs.length"
        />

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

const selectedItems = ref<Set<IPasswordMenuItem>>(new Set())
const { menu } = storeToRefs(usePasswordsActionMenuStore())

const {
  currentGroupItems,
  breadCrumbs,
  selectedGroupItems,
  currentGroupId,
  inTrashGroup,
} = storeToRefs(usePasswordGroupStore())
const { insertGroupItemsAsync } = usePasswordGroupStore()

const groupItems = computed<IPasswordMenuItem[]>(() => {
  const items: IPasswordMenuItem[] = []

  items.push(
    ...currentGroupItems.value.groups.map<IPasswordMenuItem>((group) => ({
      name: group.name,
      id: group.id,
      icon: group.icon,
      type: 'group',
      color: group.color,
    })),
  )

  items.push(
    ...currentGroupItems.value.items.map<IPasswordMenuItem>((item) => ({
      name: item.title,
      id: item.id,
      icon: item.icon,
      type: 'item',
    })),
  )
  return items
})

const { isVisible } = storeToRefs(useSidebarStore())

const onEditAsync = async () => {
  const item = selectedItems.value.values().next().value
  console.log('onEditAsync', item)
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

const { t } = useI18n()
const { add } = useSnackbar()

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
const { deleteGroupAsync, syncGroupItemsAsync } = usePasswordGroupStore()
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

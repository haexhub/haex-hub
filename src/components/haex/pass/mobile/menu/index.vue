<template>
  <ul
    class="flex flex-col w-full h-full gap-y-2 *:first:rounded-t-md *:last:rounded-b-md"
    ref="listRef"
  >
    <li
      v-for="(group, index) in groupItems.groups"
      class="bg-base-100 rounded-lg hover:bg-base-content/20 origin-to intersect:motion-preset-slide-down intersect:motion-ease-spring-bouncier intersect:motion-delay ease-in-out shadow"
      :class="{
        'bg-base-content/20 outline outline-accent hover:bg-base-content/20':
          selectedItems.has(group.id),
      }"
      :style="{ '--motion-delay': `${50 * index}ms` }"
      :key="group.id"
      v-on-long-press="[
        onLongPressCallbackHook,
        {
          delay: 1000,
        },
      ]"
    >
      <HaexPassMobileMenuGroup
        :group
        @click="onClickGroupAsync"
        class="px-4 py-2"
      />
    </li>
    <li
      v-for="item in groupItems.items"
      :key="item.id"
    >
      <HaexPassMobileMenuItem :item />
    </li>
  </ul>
</template>

<script setup lang="ts">
import type {
  SelectHaexPasswordsGroups,
  SelectHaexPasswordsItems,
} from '~~/src-tauri/database/schemas/vault'

import { vOnLongPress } from '@vueuse/components'

defineProps<{
  groupItems: {
    items: SelectHaexPasswordsItems[]
    groups: SelectHaexPasswordsGroups[]
  }
}>()

const selectedItems = ref<Set<string>>(new Set())
const longPressedHook = shallowRef(false)

const onLongPressCallbackHook = (_: PointerEvent) => {
  longPressedHook.value = true
}

const localePath = useLocalePath()
const onClickGroupAsync = async (group: SelectHaexPasswordsGroups) => {
  if (longPressedHook.value) {
    if (selectedItems.value.has(group.id)) {
      selectedItems.value.delete(group.id)
    } else {
      selectedItems.value.add(group.id)
    }
    if (!selectedItems.value.size) longPressedHook.value = false
  } else {
    await navigateTo(localePath({ name: 'passwordGroupEdit' }))
  }
}

const listRef = useTemplateRef('listRef')
onClickOutside(listRef, () => {
  selectedItems.value.clear()
  longPressedHook.value = false
})
</script>

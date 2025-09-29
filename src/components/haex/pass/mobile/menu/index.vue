<template>
  <div
    v-if="menuItems?.length"
    class="flex-1 h-full"
  >
    <ul
      ref="listRef"
      class="flex flex-col w-full h-full gap-y-2 first:rounded-t-md last:rounded-b-md p-1"
    >
      <li
        v-for="(item, index) in menuItems"
        :key="item.id"
        v-on-long-press="[
          onLongPressCallbackHook,
          {
            delay: 1000,
          },
        ]"
        class="bg-accented rounded-lg hover:bg-base-content/20 origin-to intersect:motion-preset-slide-down intersect:motion-ease-spring-bouncier intersect:motion-delay ease-in-out shadow"
        :class="{
          'bg-elevated/30 outline outline-accent hover:bg-base-content/20':
            selectedItems.has(item) ||
            (currentSelectedItem?.id === item.id &&
              longPressedHook &&
              !selectedItems.has(item)),
          'opacity-60 shadow-accent': selectedGroupItems?.some(
            (_item) => _item.id === item.id,
          ),
        }"
        :style="{ '--motion-delay': `${50 * index}ms` }"
        @mousedown="
          longPressedHook
            ? (currentSelectedItem = null)
            : (currentSelectedItem = item)
        "
      >
        <HaexPassMobileMenuItem
          v-bind="item"
          @click="onClickItemAsync(item)"
        />
      </li>
    </ul>
  </div>
  <div
    v-else
    class="flex justify-center items-center flex-1"
  >
    <UiIconNoData class="text-primary size-24 shrink-0" />
  </div>
</template>

<script setup lang="ts">
import { vOnLongPress } from '@vueuse/components'
import type { IPasswordMenuItem } from './types'

defineProps<{
  menuItems: IPasswordMenuItem[]
}>()

defineEmits(['add'])
const selectedItems = defineModel<Set<IPasswordMenuItem>>('selectedItems', {
  default: new Set(),
})

const currentSelectedItem = ref<IPasswordMenuItem | null>()

const longPressedHook = ref(false)

const onLongPressCallbackHook = (_: PointerEvent) => {
  longPressedHook.value = true
}

watch(longPressedHook, () => {
  if (!longPressedHook.value) selectedItems.value.clear()
})

watch(selectedItems, () => {
  if (!selectedItems.value.size) longPressedHook.value = false
})

const localePath = useLocalePath()
const { ctrl } = useMagicKeys()
const { search } = storeToRefs(useSearchStore())

const onClickItemAsync = async (item: IPasswordMenuItem) => {
  currentSelectedItem.value = null

  if (longPressedHook.value || selectedItems.value.size || ctrl?.value) {
    if (selectedItems.value?.has(item)) {
      selectedItems.value.delete(item)
    } else {
      selectedItems.value?.add(item)
    }

    if (!selectedItems.value.size) longPressedHook.value = false
  } else {
    if (item.type === 'group')
      await navigateTo(
        localePath({
          name: 'passwordGroupItems',
          params: {
            ...useRouter().currentRoute.value.params,
            groupId: item.id,
          },
        }),
      )
    else {
      await navigateTo(
        localePath({
          name: 'passwordItemEdit',
          params: { ...useRouter().currentRoute.value.params, itemId: item.id },
        }),
      )
    }
    search.value = ''
  }
}

const listRef = useTemplateRef('listRef')
onClickOutside(listRef, async () => {
  // needed cause otherwise the unselect is to fast for other processing like "edit selected group"
  setTimeout(() => {
    longPressedHook.value = false
  }, 50)
})

const { selectedGroupItems } = storeToRefs(usePasswordGroupStore())
</script>

<template>
  <div class="relative h-full">
    <div>
      <HaexPassMobileMenu :group-items="currentGroupItems" />

      <div
        class="fixed bottom-4 flex justify-center w-full transition-all pointer-events-none"
        :class="[isVisible ? 'left-15' : 'left-0']"
      >
        <!-- <UiButton class="btn btn-primary btn-lg btn-square rotate-45">
          <Icon name="mdi:plus" />
        </UiButton> -->
        <UiButtonAction
          class="pointer-events-auto"
          :menu
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsItems } from '~~/src-tauri/database/schemas/vault'

definePageMeta({
  name: 'passwordGroupItems',
})

const { menu } = storeToRefs(usePasswordsActionMenuStore())
const items = ref<SelectHaexPasswordsItems[]>([])

const { readGroupItemsAsync } = usePasswordGroupStore()
const { currentGroupItems } = storeToRefs(usePasswordGroupStore())
const { isVisible } = storeToRefs(useSidebarStore())

console.log('currentGroupItems', currentGroupItems.value)
const test = () => console.log('currentGroupItems', currentGroupItems.value)

onMounted(async () => {})
</script>

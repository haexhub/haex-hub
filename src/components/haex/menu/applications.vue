<template>
  <UPopover v-model:open="open">
    <UButton
      icon="material-symbols:apps"
      color="neutral"
      variant="outline"
      v-bind="$attrs"
      size="xl"
    />

    <template #content>
      <ul
        class="p-4 max-h-96 grid grid-cols-3 gap-2 overflow-scroll"
        @click="open = false"
      >
        <UiButton
          v-for="item in menu"
          :key="item.id"
          square
          size="xl"
          variant="ghost"
          :ui="{
            base: 'size-24 flex flex-wrap  text-sm items-center justify-center overflow-visible',
            leadingIcon: 'size-10',
            label: 'w-full',
          }"
          :icon="item.icon"
          :label="item.name"
          :tooltip="item.name"
          @click="item.onSelect"
        />
        <!-- <UiButton
          v-for="item in extensionLinks"
          :key="item.id"
          v-bind="item"
          icon-type="svg"
        /> -->
      </ul>
    </template>
  </UPopover>
</template>

<script setup lang="ts">
//const { extensionLinks } = storeToRefs(useExtensionsStore())
const { menu } = storeToRefs(useSidebarStore())

const open = ref(false)
</script>

<i18n lang="yaml">
de:
  settings: 'Einstellungen'
  close: 'Vault schließen'

en:
  settings: 'Settings'
  close: 'Close Vault'
</i18n>

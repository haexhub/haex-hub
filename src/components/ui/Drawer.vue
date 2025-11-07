<template>
  <UDrawer
    v-bind="$attrs"
    :ui="{
      content:
        'pb-[env(safe-area-inset-bottom)] pt-[env(safe-area-inset-top)] w-dvw max-w-md sm:max-w-fit',
      ...(ui || {}),
    }"
  >
    <template
      v-for="(_, name) in $slots"
      #[name]="slotData"
    >
      <slot
        :name="name"
        v-bind="slotData"
      />
    </template>
  </UDrawer>
</template>

<script setup lang="ts">
import type { DrawerProps } from '@nuxt/ui'

/**
 * Wrapper around UDrawer that automatically applies safe area insets for mobile devices.
 * Passes through all props and slots to UDrawer.
 */
defineProps</* @vue-ignore */ DrawerProps>()
</script>

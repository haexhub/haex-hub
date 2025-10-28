<template>
  <UApp :locale="locales[locale]">
    <div data-vaul-drawer-wrapper>
      <NuxtPage />
    </div>
  </UApp>
</template>

<script setup lang="ts">
import * as locales from '@nuxt/ui/locale'
const { locale } = useI18n()

// Handle Android back button
useAndroidBackButton()
</script>

<style>
.fade-enter-active {
  transition: all 1s ease-out;
}

.fade-leave-active {
  transition: all 1s ease-out reverse;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

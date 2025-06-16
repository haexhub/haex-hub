<template>
  <div>
    <NuxtLayout>
      <NuxtPage />
      <NuxtSnackbar />
    </NuxtLayout>
  </div>
</template>

<script setup lang="ts">
const { currentThemeValue } = storeToRefs(useUiStore())

useHead({
  htmlAttrs: {
    'data-theme': currentThemeValue,
  },
})
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

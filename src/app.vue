<template>
  <div>
    <NuxtLayout :data-theme="currentTheme.value">
      <NuxtPage />
      <NuxtSnackbar />
    </NuxtLayout>
  </div>
</template>

<script setup lang="ts">

const { currentTheme } = storeToRefs(useUiStore())
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

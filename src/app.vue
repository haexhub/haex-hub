<template>
  <div class=" ">
    <BaseDialog />
    <!-- <NuxtLayout>
      <NuxtPage />
      <Toaster />
    </NuxtLayout> -->
  </div>
</template>

<script setup lang="ts">
//import 'vue-sonner/style.css'
//const { currentTheme } = storeToRefs(useUiStore())
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

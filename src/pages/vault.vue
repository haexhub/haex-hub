<template>
  <div class="w-full h-full">
    <NuxtLayout name="app">
      <NuxtPage />
    </NuxtLayout>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  middleware: 'database',
})
</script>

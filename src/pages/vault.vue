<template>
  <div class="h-full w-full">
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

<template>
  <div class="p-4">
    <NuxtPage />
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'passwords',
})

const { currentGroupId } = storeToRefs(usePasswordGroupStore())
</script>

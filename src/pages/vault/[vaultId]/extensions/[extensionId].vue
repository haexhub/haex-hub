<template>
  <div>
    hier kommt die erweiterung
    {{ useRouter().currentRoute.value.params.extensionId }}
    <iframe></iframe>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'haexExtension',
});
</script>

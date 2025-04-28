<template>
    <div class="text-white">
        <NuxtLayout name="app">
            <NuxtPage />
        </NuxtLayout>
    </div>
</template>

<script setup lang="ts">
const { createTable } = useVaultStore();
</script>

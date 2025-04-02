<template>
  <div class="tab-bar">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="tab"
      :class="{ active: tab.id === activeTabId }"
      @click="$emit('activateTab', tab.id)"
    >
      <span class="tab-title">
        {{ tab.title || 'Neuer Tab' }}
      </span>
      <button
        class="tab-close"
        @click.stop="$emit('closeTab', tab.id)"
      >
        ×
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Tab {
  id: string;
  title: string;
  url: string;
  isLoading: boolean;
  isActive: boolean;
}

interface Props {
  tabs: Tab[];
  activeTabId: string | null;
}

defineProps<Props>();

defineEmits<{
  (e: 'closeTab', tabId: string): void;
  (e: 'activateTab', tabId: string): void;
}>();
</script>

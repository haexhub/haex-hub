<template>
  <div
    class="tree-view-selected:bg-base-200/60 dragged:bg-primary/20 dragged:rounded nested-4 cursor-pointer rounded-md px-2"
    role="treeitem" :data-tree-view-item="JSON.stringify({
      value,
      isDir: false,
    })
      ">
    <div class="flex items-center gap-x-3">
      <span class="icon-[tabler--file] text-base-content size-4 flex-shrink-0"/>
      <div class="grow">
        <span class="text-base-content">{{ value }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps({
  value: String,
});

const id = useId();
const controlId = useId();
const isActive = ref(false);
</script>
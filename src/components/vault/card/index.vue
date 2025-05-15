<template>
  <div class="card">
    <slot name="image" />

    <div class="card-header">
      <div v-if="$slots.title || title">
        <Icon :name="icon" />
        <h5 v-if="title" class="card-title mb-0">
          {{ title }}
        </h5>
        <slot v-else name="title" />
      </div>
      <div class="text-base-content/50">Your journey starts here</div>
    </div>

    <div class="card-body">
      <slot />
      aaaaaaaaa
      <div class="card-actions" v-if="$slots.action">
        <slot name="action" />
      </div>
    </div>
  </div>
  <!-- <div class="bg-base-100 w-full mx-auto shadow h-full overflow-hidden pt-[7.5rem]">
    <div
      class="fixed top-0 right-0 z-10 transition-all duration-700 w-full font-semibold text-lg h-[7.5rem]"
    >
      <div
        class="justify-center items-center flex flex-wrap border-b rounded-b border-secondary h-full"
      >
        <slot name="header" />
      </div>
    </div>

    <div class="h-full overflow-scroll bg-base-200">
      <slot />
    </div>
  </div> -->
</template>

<script setup lang="ts">
const emit = defineEmits(["close", "submit"]);

defineProps<{ title?: string; icon?: string }>();

const { escape, enter } = useMagicKeys();

watchEffect(async () => {
  if (escape.value) {
    await nextTick();
    emit("close");
  }
});

watchEffect(async () => {
  if (enter.value) {
    await nextTick();
    emit("submit");
  }
});
</script>

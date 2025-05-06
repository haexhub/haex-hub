<template>
  <div class="accordion divide-neutral/20 divide-y">
    <div class="accordion-item active" :id="itemId" ref="accordionRef">
      <button
        class="accordion-toggle inline-flex items-center gap-x-4 text-start"
        :aria-controls="collapseId"
        aria-expanded="true"
        type="button"
      >
        <span
          class="icon-[tabler--chevron-right] accordion-item-active:rotate-90 size-5 shrink-0 transition-transform duration-300 rtl:rotate-180"
        ></span>
        <slot name="title" />
      </button>
      <div
        :id="collapseId"
        class="accordion-content w-full overflow-hidden transition-[height] duration-300"
        :aria-labelledby="itemId"
        role="region"
      >
        <div class="px-5 pb-4">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { HSAccordion } from "flyonui/flyonui";

const itemId = useId();
const collapseId = useId();

const accordionRef = useTemplateRef("accordionRef");
const accordion = ref<HSAccordion>();

onMounted(() => {
  if (accordionRef.value) {
    accordion.value = new HSAccordion(accordionRef.value);
  }
});
</script>

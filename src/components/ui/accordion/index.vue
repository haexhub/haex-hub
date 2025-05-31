<template>
  <div
    class="accordion divide-neutral/20 divide-y accordion-shadow *:accordion-item-active:shadow-md"
  >
    <div
      :id="itemId"
      ref="accordionRef"
      class="accordion-item active"
    >
      <button
        class="accordion-toggle inline-flex items-center gap-x-4 text-start"
        :aria-controls="collapseId"
        aria-expanded="true"
        type="button"
      >
        <span
          class="icon-[tabler--chevron-right] accordion-item-active:rotate-90 size-5 shrink-0 transition-transform duration-300 rtl:rotate-180"
        />
        <slot name="title" />
      </button>
      <div
        :id="collapseId"
        class="accordion-content w-full overflow-hidden transition-[height] duration-300"
        :aria-labelledby="itemId"
        role="region"
      >
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { HSAccordion } from 'flyonui/flyonui'

const itemId = useId()
const collapseId = useId()

const accordionRef = useTemplateRef('accordionRef')
const accordion = ref<HSAccordion>()

onMounted(() => {
  if (accordionRef.value) {
    accordion.value = new window.HSAccordion(accordionRef.value)
    accordion.value.hide()
  }
})
</script>

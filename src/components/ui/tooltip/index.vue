<template>
  <div class="tooltip [--prevent-popper:false]">
    <div
      class="tooltip-toggle"
      :aria-label="tooltip"
    >
      <slot>
        <button class="btn btn-square">
          <Icon name="mdi:chevron-up-box-outline" />
        </button>
      </slot>

      <span
        class="tooltip-content tooltip-shown:opacity-100 tooltip-shown:visible pointer-events-none z-50"
        role="tooltip"
      >
        <span class="tooltip-body">
          {{ tooltip }}
        </span>
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PropType } from 'vue'

const props = defineProps({
  direction: {
    type: String as PropType<
      | 'top'
      | 'top-start'
      | 'top-end'
      | 'bottom'
      | 'bottom-start'
      | 'bottom-end'
      | 'right'
      | 'right-start'
      | 'right-end'
      | 'left'
      | 'left-start'
      | 'left-end'
    >,
    default: 'top',
  },

  tooltip: {
    type: String,
    default: '',
  },

  trigger: {
    type: String as PropType<'focus' | 'hover' | 'click'>,
    default: 'hover',
  },
})
</script>

<template>
  <div class="dropdown relative inline-flex" :class="dropdownClass">
    <button
      :id
      class="dropdown-toggle"
      :class="activatorClass"
      aria-haspopup="menu"
      aria-expanded="false"
      :aria-label="label"
    >
      <slot name="activator">
        <slot name="label">
          {{ label }}
        </slot>
        <span
          class="icon-[tabler--chevron-down] dropdown-open:rotate-180 size-4"
        >
        </span>
      </slot>
    </button>

    <slot name="items" :items>
      <ul
        class="dropdown-menu dropdown-open:opacity-100 hidden min-w-28"
        role="menu"
        aria-orientation="vertical"
        :aria-labelledby="id"
      >
        <component
          :is="itemIs"
          class="dropdown-item"
          v-for="item in items"
          @click="$emit('select', item)"
        >
          <slot name="item" :item>
            {{ item }}
          </slot>
        </component>
      </ul>
    </slot>
  </div>
</template>

<script setup lang="ts" generic="T">
const { itemIs = 'li' } = defineProps<{
  label?: string
  items?: T[]
  itemIs?: string
  activatorClass?: string
  dropdownClass?: string
}>()

defineEmits<{ select: [T] }>()
const id = useId()
</script>

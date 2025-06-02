<template>
  <div
    class="dropdown relative inline-flex"
    :class="offset"
  >
    <button
      :id
      class="dropdown-toggle"
      v-bind="$attrs"
      aria-haspopup="menu"
      aria-expanded="false"
      :aria-label="label"
    >
      <slot name="activator">
        {{ label }}
        <span
          class="icon-[tabler--chevron-down] dropdown-open:rotate-180 size-4"
        />
      </slot>
    </button>

    <ul
      class="dropdown-menu dropdown-open:opacity-100 hidden min-w-28"
      role="menu"
      aria-orientation="vertical"
      :aria-labelledby="id"
    >
      <slot name="items">
        <li
          :is="itemIs"
          v-for="item in items"
          class="dropdown-item"
          @click="$emit('select', item)"
        >
          <slot
            name="item"
            :item
          >
            {{ item }}
          </slot>
        </li>
      </slot>
    </ul>
  </div>
</template>

<script setup lang="ts" generic="T">
const { itemIs = 'li', offset = '[--offset:0]' } = defineProps<{
  label?: string
  items?: T[]
  itemIs?: string
  activatorClass?: string
  offset?: string
}>()

defineOptions({
  inheritAttrs: false,
})

defineEmits<{ select: [T] }>()

const id = useId()

//const offset = '[--offset:30]'
</script>

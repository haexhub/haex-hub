<template>
  <div
    :class="offset"
    class="dropdown relative inline-flex"
  >
    <button
      :aria-label="label"
      :id
      aria-expanded="false"
      aria-haspopup="menu"
      class="dropdown-toggle"
      type="button"
      v-bind="$attrs"
    >
      <slot name="activator">
        {{ label }}
        <span
          class="icon-[tabler--chevron-down] dropdown-open:rotate-180 size-4"
        />
      </slot>
    </button>

    <ul
      :aria-labelledby="id"
      aria-orientation="vertical"
      class="dropdown-menu dropdown-open:opacity-100 hidden min-w-28 z-20 shadow shadow-primary"
      role="menu"
    >
      <slot
        name="items"
        :items
      >
        <li
          :is="itemIs"
          @click="read_only ? '' : $emit('select', item)"
          class="dropdown-item"
          v-for="item in items"
        >
          <slot
            :item
            name="item"
          >
            {{ item }}
          </slot>
        </li>
      </slot>
    </ul>
  </div>
</template>

<script setup lang="ts" generic="T">
defineOptions({
  inheritAttrs: false,
})

const { itemIs = 'li', offset = '[--offset:0]' } = defineProps<{
  label?: string
  items?: T[]
  itemIs?: string
  activatorClass?: string
  offset?: string
  read_only?: boolean
}>()

defineEmits<{ select: [T] }>()

const id = useId()

//const offset = '[--offset:30]'
</script>

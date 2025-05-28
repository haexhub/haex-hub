<template>
  <div class="dropdown relative inline-flex">
    <button :id class="dropdown-toggle" v-bind="$attrs" aria-haspopup="menu" aria-expanded="false" :aria-label="label">
      <slot name="activator">
        {{ label }}
        <span class="icon-[tabler--chevron-down] dropdown-open:rotate-180 size-4"/>
      </slot>
    </button>

    <ul
class="dropdown-menu dropdown-open:opacity-100 hidden min-w-28" role="menu" aria-orientation="vertical"
      :aria-labelledby="id">

      <slot name="items">


        <li :is="itemIs" v-for="item in items" class="dropdown-item" @click="$emit('select', item)">
          <slot name="item" :item>
            {{ item }}
          </slot>
        </li>
      </slot>


    </ul>
  </div>
</template>

<script setup lang="ts" generic="T">
const { itemIs = "li" } = defineProps<{
  label?: string;
  items?: T[];
  itemIs?: string;
  activatorClass?: string;
}>();

defineOptions({
  inheritAttrs: false,
});

defineEmits<{ select: [T] }>();

const id = useId();
</script>
<template>
  <div
    ref="activator"
    class="relative advance-select flex w-full input-group"
  >
    <button
      :id
      class="advance-select-toogle flex justify-between grow p-3"
      @click.prevent="toogleMenu"
      :disabled="read_only"
    >
      <slot
        name="value"
        :value
        class=""
      >
        <span>{{ value ?? label }}</span>
      </slot>
    </button>
    <button
      @click.prevent="toogleMenu"
      class="flex items-center p-2 hover:shadow rounded-md hover:bg-primary hover:text-base-content"
      :disabled="read_only"
    >
      <i class="i-[material-symbols--keyboard-arrow-down] size-4" />
    </button>
    <!--  <div data-select-dropdown="" class="absolute advance-select-menu max-h-44 top-full opened" role="listbox" tabindex="-1" aria-orientation="vertical" style="margin-top: 10px;"><div data-value="dark" data-title-value="Dunkel" tabindex="0" class="cursor-pointer advance-select-option selected:active" data-id="0"><div><div class="flex items-center"> <div class="me-2" data-icon=""><icon name="undefined" class="flex-shrink-0 size-4 text-base-content mt-1 max-w-full"></icon></div> <div class="font-semibold text-base-content" data-title="">Dunkel</div> </div> <div class="mt-1.5 text-sm text-base-content/80" data-description=""></div> </div></div><div data-value="light" data-title-value="Hell" tabindex="1" class="cursor-pointer advance-select-option selected:active" data-id="1"><div><div class="flex items-center"> <div class="me-2" data-icon=""><icon name="undefined" class="flex-shrink-0 size-4 text-base-content mt-1 max-w-full"></icon></div> <div class="font-semibold text-base-content" data-title="">Hell</div> </div> <div class="mt-1.5 text-sm text-base-content/80" data-description=""></div> </div></div><div data-value="soft" data-title-value="Soft" tabindex="2" class="cursor-pointer advance-select-option selected:active selected" data-id="2"><div><div class="flex items-center"> <div class="me-2" data-icon=""><icon name="undefined" class="flex-shrink-0 size-4 text-base-content mt-1 max-w-full"></icon></div> <div class="font-semibold text-base-content" data-title="">Soft</div> </div> <div class="mt-1.5 text-sm text-base-content/80" data-description=""></div> </div></div></div>
class="absolute advance-select-menu max-h-44 top-full opened" -->
    <!-- Dropdown menu -->
    <ul
      data-select-dropdown
      classaaa="advance-select-menu bg-white divide-y divide-slate-100 rounded-lg shadow dark:bg-slate-700 absolute top-12"
      :class="{ hidden: !show }"
      class="absolute advance-select-menu max-h-44 top-full opened"
      role="listbox"
      tabindex="-1"
      aria-orientation="vertical"
    >
      <!-- <ul
        class=""
        :aria-labelledby="id"
      > -->
      <li
        v-for="(option, index) in options"
        :key="index"
        class="advance-select-option selected:active font-semibold text-base-content"
        @click=";(value = option), (show = false)"
      >
        <slot
          name="option"
          :option
        >
          {{ option }}
        </slot>
      </li>
      <!--  </ul> -->
    </ul>
  </div>
</template>

<script setup lang="ts" generic="T">
import { onClickOutside } from '@vueuse/core'

const id = useId()

defineProps({
  label: String,
  options: {
    type: Array as PropType<T[]>,
    default: () => [],
  },
  read_only: Boolean,
})
const value = defineModel<T>()

const show = ref(false)
const toogleMenu = () => {
  show.value = !show.value
}

const activator = ref(null)

onClickOutside(activator, () => (show.value = false))
</script>

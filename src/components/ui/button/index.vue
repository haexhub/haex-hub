<template>
  <div>
    <UTooltip :text="buttonProps?.tooltip">
      <UButton
        class="pointer-events-auto"
        v-bind="{
          ...{ size: isSmallScreen ? 'lg' : 'md' },
          ...buttonProps,
          ...$attrs,
        }"
        @click="(e) => $emit('click', e)"
      >
        <template
          v-for="(_, slotName) in $slots"
          #[slotName]="slotProps"
        >
          <slot
            :name="slotName"
            v-bind="slotProps"
          />
        </template>
      </UButton>
    </UTooltip>
  </div>
</template>

<script setup lang="ts">
import type { ButtonProps } from '@nuxt/ui'

interface IButtonProps extends /* @vue-ignore */ ButtonProps {
  tooltip?: string
}
const buttonProps = defineProps<IButtonProps>()
defineEmits<{ click: [Event] }>()

const { isSmallScreen } = storeToRefs(useUiStore())
</script>

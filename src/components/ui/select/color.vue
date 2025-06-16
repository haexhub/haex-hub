<template>
  <div class="flex items-center gap-4 relative">
    <UiButton
      :style="{ 'background-color': model }"
      :class="[textColorClass]"
      @click="colorRef?.click()"
    >
      {{ t('label') }}
    </UiButton>

    <input
      :id
      :readonly="read_only"
      :disabled="read_only"
      :title="t('pick')"
      class="top-0 left-0 absolute size-0"
      type="color"
      v-model="model"
      ref="colorRef"
    />

    <UiTooltip :tooltip="t('reset')">
      <button
        @click="model = ''"
        class="btn btn-sm text-sm btn-outline btn-error"
        :class="{ 'btn-disabled': read_only }"
        type="button"
      >
        <Icon name="mdi:refresh" />
      </button>
    </UiTooltip>
  </div>
</template>

<script setup lang="ts">
const id = useId()
const { t } = useI18n()

const model = defineModel<string | null>()
const colorRef = useTemplateRef('colorRef')
defineProps({
  read_only: Boolean,
})

const { currentTheme } = storeToRefs(useUiStore())
const textColorClass = computed(() => {
  if (!model.value)
    return currentTheme.value.value === 'dark' ? 'text-black' : 'text-white'

  const color = getContrastingTextColor(model.value)
  return color === 'white' ? 'text-white' : 'text-black'
})
</script>

<i18n lang="json">
{
  "de": {
    "label": "Farbauswahl",
    "title": "Wähle eine Farbe aus",
    "reset": "zurücksetzen",
    "pick": "Auswahl"
  },
  "en": {
    "label": "Color Picker",
    "title": "Choose a color",
    "reset": "Reset",
    "pick": "Pick"
  }
}
</i18n>

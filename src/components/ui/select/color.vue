<template>
  <div class="flex flex-wrap items-center gap-4 relative">
    <UiButton
      :style="{ 'background-color': model }"
      :class="[textColorClass]"
      @click="colorRef?.click()"
    >
      {{ t('label') }}
    </UiButton>

    <input
      :id
      ref="colorRef"
      v-model="model"
      :readonly="readOnly"
      :disabled="readOnly"
      :title="t('pick')"
      class="top-0 left-0 absolute size-0"
      type="color"
    />

    <UiTooltip :tooltip="t('reset')">
      <UiButton
        color="error"
        :class="{ 'btn-disabled': readOnly }"
        icon="mdi:refresh"
        :disabled="readOnly"
        @click="model = ''"
      />
    </UiTooltip>
  </div>
</template>

<script setup lang="ts">
const id = useId()
const { t } = useI18n()

const model = defineModel<string | null>()
const colorRef = useTemplateRef('colorRef')
defineProps({
  readOnly: Boolean,
})

const { currentTheme } = storeToRefs(useUiStore())
const textColorClass = computed(() => {
  if (!model.value && currentTheme.value)
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

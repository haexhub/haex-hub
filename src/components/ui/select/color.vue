<template>
  <div class="flex items-center gap-4">
    <label
      :for="id"
      class="font-medium"
    >
      {{ t('label') }}
    </label>

    <input
      :id
      :readonly="read_only"
      :disabled="read_only"
      :title="t('title')"
      class="p-0 cursor-pointer disabled:opacity-50 disabled:pointer-events-none w-14 h-10"
      type="color"
      v-model="model"
    />

    <button
      @click="model = null"
      class="btn btn-sm text-sm"
      :class="{ 'btn-disabled': read_only }"
    >
      {{ t('reset') }}
    </button>
  </div>
</template>

<script setup lang="ts">
const id = useId()
const { t } = useI18n()

const model = defineModel()

defineProps({
  read_only: Boolean,
})
</script>

<i18n lang="json">
{
  "de": {
    "label": "Farbauswahl",
    "title": "Wähle eine Farbe aus",
    "reset": "zurücksetzen"
  },
  "en": {
    "label": "Color Picker",
    "title": "Choose a color",
    "reset": "Reset"
  }
}
</i18n>

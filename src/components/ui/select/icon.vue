<template>
  <UiSelect
    v-model="icon"
    :options="icons"
    label="Icon Picker"
  >
    <template #value="{ value }">
      <Icon
        :name="value"
        v-if="value"
      />
    </template>
    <template #option="{ option }">
      <Icon :name="option ?? ''" />
    </template>
  </UiSelect>

  <UiDropdown :items="icons">
    <template #activator> {{ icons.find((_icon) => _icon === icon) }}</template>
    <template #item="{ item }">
      <Icon :name="`mdi:${item}`" />
      {{ item }}
    </template>
  </UiDropdown>
</template>

<script setup lang="ts">
const icons = [
  'streamline:money-bank-institution-money-saving-bank-payment-finance',
  'material-symbols:star-outline-rounded',
  'pepicons-pop:smartphone-home-button',
  'majesticons:desktop-computer-line',
  'mdi:folder',
]

const icon = defineModel<string | undefined | null>({ default: '' })
</script>

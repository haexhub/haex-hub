<template>
  <UiAccordion v-if="filesystem?.read?.length">
    <template #title>
      <h3>{{ t('permission.read') }}</h3>
    </template>
    <ul class="space-y-0.5">
      <li v-for="read in filesystem?.read" class="flex items-center justify-between px-4 py-0.5">
        <div class="flex items-center gap-1">
          <input :id="Object.keys(read).at(0)" type="checkbox" class="checkbox" :checked="Object.values(read).at(0)" >
          <label class="label-text text-base" :for="Object.keys(read).at(0)">{{
            Object.keys(read).at(0)
          }}</label>
        </div>
      </li>
    </ul>
  </UiAccordion>

  <UiAccordion v-if="filesystem?.write?.length">
    <template #title>
      <h3>{{ t('permission.write') }}</h3>
    </template>

    <ul class="space-y-0.5">
      <li v-for="write in filesystem?.write" class="flex items-center justify-between px-4 py-0.5">
        <div class="flex items-center gap-1">
          <input
:id="Object.keys(write).at(0)" type="checkbox" class="checkbox"
            :checked="Object.values(write).at(0)" >
          <label class="label-text text-base" :for="Object.keys(write).at(0)">{{
            Object.keys(write).at(0)
          }}</label>
        </div>
      </li>
    </ul>
  </UiAccordion>
</template>

<script setup lang="ts">
defineProps<{
  filesystem?: {
    read?: Record<string, boolean>[]
    write?: Record<string, boolean>[]
  }
}>()
const { t } = useI18n()
</script>

<i18n lang="yaml">
  de: 
    permission: 
      read: Lesen
      write: Schreiben

  en: 
    permission: 
      read: Read
      write: Write
</i18n>

<template>
  <UAccordion v-if="database?.read?.length">
    <template #title>
      <h3>{{ t('permission.read') }}</h3>
    </template>

    <ul class="space-y-0.5">
      <li
        v-for="read in database?.read"
        class="flex items-center justify-between px-4 py-1"
      >
        <div class="flex items-center gap-1">
          <input
            :id="Object.keys(read).at(0)"
            type="checkbox"
            class="checkbox"
            :checked="Object.values(read).at(0)"
          >
          <label
            class="label-text text-base"
            :for="Object.keys(read).at(0)"
            >{{ Object.keys(read).at(0) }}</label
          >
        </div>
      </li>
    </ul>
  </UAccordion>

  <UAccordion v-if="database?.write?.length">
    <template #title>
      <h3>{{ t('permission.write') }}</h3>
    </template>

    <ul class="space-y-0.5">
      <li
        v-for="write in database?.write"
        class="flex items-center justify-between px-4 py-0.5"
      >
        <div class="flex items-center gap-1">
          <input
            :id="Object.keys(write).at(0)"
            type="checkbox"
            class="checkbox"
            :checked="Object.values(write).at(0)"
          >
          <label
            class="label-text text-base"
            :for="Object.keys(write).at(0)"
            >{{ Object.keys(write).at(0) }}</label
          >
        </div>
      </li>
    </ul>
  </UAccordion>

  <UAccordion v-if="database?.create?.length">
    <template #title>
      <h3>{{ t('permission.create') }}</h3>
    </template>

    <ul class="space-y-0.5">
      <li
        v-for="create in database?.create"
        class="flex items-center justify-between px-4 py-0.5"
      >
        <div class="flex items-center gap-1">
          <input
            :id="Object.keys(create).at(0)"
            type="checkbox"
            class="checkbox"
            :checked="Object.values(create).at(0)"
          >
          <label
            class="label-text text-base"
            :for="Object.keys(create).at(0)"
            >{{ Object.keys(create).at(0) }}</label
          >
        </div>
      </li>
    </ul>
  </UAccordion>
</template>

<script setup lang="ts">
defineProps<{
  database?: {
    read?: Record<string, boolean>[]
    write?: Record<string, boolean>[]
    create?: Record<string, boolean>[]
  }
}>()
const { t } = useI18n()
</script>

<i18n lang="yaml">
de:
  permission:
    read: Lesen
    write: Schreiben
    create: Erstellen

en:
  permission:
    read: Read
    write: Write
    create: Create
</i18n>

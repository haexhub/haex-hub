<template>
  <UAccordion>
    <template #title>
      <h3>{{ t('http.access') }}</h3>
    </template>

    <ul class="space-y-0.5">
      <li
        v-for="access in http"
        class="flex items-center justify-between px-4 py-0.5"
      >
        <div class="flex items-center gap-1">
          <input
            :id="Object.keys(access).at(0)"
            type="checkbox"
            class="checkbox"
            :checked="Object.values(access).at(0)"
          >
          <label
            class="label-text text-base"
            :for="Object.keys(access).at(0)"
            >{{ Object.keys(access).at(0) }}</label
          >
        </div>
      </li>
    </ul>
  </UAccordion>
</template>

<script setup lang="ts">
defineProps<{ http?: Record<string, boolean>[] }>()
const { t } = useI18n()
</script>

<i18n lang="yaml">
de:
  http:
    access: Internet Zugriff

en:
  http:
    access: Internet Access
</i18n>

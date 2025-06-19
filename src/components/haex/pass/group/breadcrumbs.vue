<template>
  <div class="breadcrumbs">
    <ul>
      <li>
        <NuxtLinkLocale :to="{ name: 'passwordGroupItems' }">
          <Icon
            name="mdi:safe"
            size="24"
          />
        </NuxtLinkLocale>
      </li>
      <template v-for="item in items">
        <li class="breadcrumbs-separator rtl:rotate-180">
          <Icon name="tabler:chevron-right" />
        </li>

        <li>
          <NuxtLinkLocale
            :to="{ name: 'passwordGroupItems', params: { groupId: item.id } }"
          >
            {{ item.name }}
          </NuxtLinkLocale>
        </li>
      </template>
      <li class="ml-2">
        <NuxtLinkLocale
          :to="{
            name: 'passwordGroupEdit',
            params: { groupId: lastGroup?.id },
          }"
        >
          <Icon name="mdi:pencil" />
        </NuxtLinkLocale>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

const groups = defineProps<{ items: SelectHaexPasswordsGroups[] }>()

const lastGroup = computed(() => groups.items.at(-1))
</script>

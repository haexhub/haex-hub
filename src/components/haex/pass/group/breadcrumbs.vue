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
        <UiTooltip
          :tooltip="t('edit')"
          class="[--placement:bottom]"
        >
          <NuxtLinkLocale
            :to="{
              name: 'passwordGroupEdit',
              params: { groupId: lastGroup?.id },
            }"
          >
            <Icon name="mdi:pencil" />
          </NuxtLinkLocale>
        </UiTooltip>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { UiTooltip } from '#components'
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

const groups = defineProps<{ items: SelectHaexPasswordsGroups[] }>()

const lastGroup = computed(() => groups.items.at(-1))

const { t } = useI18n()
</script>

<i18n lang="yaml">
de:
  edit: Bearbeiten

en:
  edit: Edit
</i18n>

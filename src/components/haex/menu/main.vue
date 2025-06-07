<template>
  <UiDropdown offset="[--offset:20]">
    <template #activator>
      <div
        class="size-9.5 rounded-full items-center justify-center text-base-content text-base"
      >
        <Icon
          name="mdi:format-list-bulleted"
          class="size-full p-2"
        />
      </div>
    </template>

    <!-- <ul class="dropdown-menu dropdown-open:opacity-100 hidden min-w-60" role="menu" aria-orientation="vertical"
      aria-labelledby="dropdown-avatar"> -->

    <template #items>
      <li>
        <NuxtLinkLocale
          class="dropdown-item"
          :to="{ name: 'settings' }"
        >
          <span class="icon-[tabler--settings]" />
          {{ t('settings') }}
        </NuxtLinkLocale>
      </li>

      <li class="dropdown-footer gap-2">
        <button
          class="btn btn-error btn-soft btn-block"
          @click="onVaultCloseAsync"
        >
          <span class="icon-[tabler--logout]" />
          {{ t('vault.close') }}
        </button>
      </li>
    </template>
    <!-- 
    </ul> -->
  </UiDropdown>
</template>

<script setup lang="ts">
const { t } = useI18n()
const { closeAsync } = useVaultStore()

const onVaultCloseAsync = async () => {
  await closeAsync()
  await navigateTo(useLocalePath()({ name: 'vaultOpen' }))
}
</script>

<i18n lang="yaml">
de:
  settings: 'Einstellungen'
  vault:
    close: 'Vault schließen'

en:
  settings: 'Settings'
  vault:
    close: 'Close Vault'
</i18n>

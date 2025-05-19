<template>
  <UiDropdown activator-class="btn btn-text btn-circle">
    <template #activator>
      <div
        class="size-9.5 rounded-full items-center justify-center text-base-content text-base"
      >
        <Icon name="mdi:format-list-bulleted" class="size-full p-2" />
      </div>
    </template>

    <template #items>
      <ul
        class="dropdown-menu dropdown-open:opacity-100 hidden min-w-60"
        role="menu"
        aria-orientation="vertical"
        aria-labelledby="dropdown-avatar"
      >
        <li>
          <NuxtLinkLocale class="dropdown-item" :to="{ name: 'haexSettings' }">
            <span class="icon-[tabler--settings]"></span>
            {{ t('settings') }}
          </NuxtLinkLocale>
        </li>

        <li class="dropdown-footer gap-2">
          <button
            class="btn btn-error btn-soft btn-block"
            @click="onVaultCloseAsync"
          >
            <span class="icon-[tabler--logout]"></span>
            {{ t('vault.close') }}
          </button>
        </li>
      </ul>
    </template>
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

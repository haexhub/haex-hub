<template>
  <UDropdownMenu :items>
    <UButton
      icon="mdi:menu"
      color="neutral"
      variant="outline"
    />
  </UDropdownMenu>
</template>

<script setup lang="ts">
import type { DropdownMenuItem } from '@nuxt/ui'

const { t } = useI18n()
const { closeAsync } = useVaultStore()

const onVaultCloseAsync = async () => {
  await closeAsync()
  await navigateTo(useLocalePath()({ name: 'vaultOpen' }))
}

const items: DropdownMenuItem[] = [
  {
    icon: 'tabler:settings',
    label: t('settings'),
    to: useLocalePath()({ name: 'settings' }),
  },
  {
    icon: 'tabler:logout',
    label: t('close'),
    onSelect: () => onVaultCloseAsync(),
    color: 'error',
  },
]
</script>

<i18n lang="yaml">
de:
  settings: 'Einstellungen'
  close: 'Vault schließen'

en:
  settings: 'Settings'
  close: 'Close Vault'
</i18n>

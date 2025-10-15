<template>
  <UDropdownMenu :items>
    <UButton
      icon="mdi:menu"
      color="neutral"
      variant="outline"
      v-bind="$attrs"
      size="xl"
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
    icon: 'mdi:code-braces',
    label: t('developer'),
    to: useLocalePath()({ name: 'settings-developer' }),
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
  developer: 'Entwickler'
  close: 'Vault schließen'

en:
  settings: 'Settings'
  developer: 'Developer'
  close: 'Close Vault'
</i18n>

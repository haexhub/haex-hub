<template>
  <div
    class="grid grid-rows-2 sm:grid-cols-2 sm:gap-2 p-2 max-w-2xl w-full h-fit"
  >
    <div class="p-2">{{ t('language') }}</div>
    <div><UiDropdownLocale @select="onSelectLocaleAsync" /></div>

    <div class="p-2">{{ t('design') }}</div>
    <div><UiDropdownTheme @select="onSelectThemeAsync" /></div>

    <div class="p-2">{{ t('vaultName') }}</div>
    <div>
      <UiInput v-model="currentVaultName" :placeholder="t('vaultName')">
        <template #append>
          <UiTooltip :tooltip="t('save')">
            <UiButton class="btn-primary" @click="onSetVaultNameAsync">
              <Icon name="mdi:content-save-outline" />
            </UiButton>
          </UiTooltip>
        </template>
      </UiInput>
    </div>
  </div>
</template>

<script setup lang="ts">
import { eq } from 'drizzle-orm'
import { type Locale } from 'vue-i18n'
import { haexSettings } from '~~/src-tauri/database/schemas/vault'

definePageMeta({
  name: 'haexSettings',
})

const { t, setLocale } = useI18n()

const { currentVault, currentVaultName } = storeToRefs(useVaultStore())
const { updateVaultNameAsync } = useVaultStore()

const onSelectLocaleAsync = async (locale: Locale) => {
  console.log('onSelectLocaleAsync', locale)
  const update = await currentVault.value?.drizzle
    .update(haexSettings)
    .set({ key: 'locale', value: locale })
    .where(eq(haexSettings.key, 'locale'))
  await setLocale(locale)
  console.log('update locale', update)
}

const { currentTheme } = storeToRefs(useUiStore())

const onSelectThemeAsync = async (theme: ITheme) => {
  const update = await currentVault.value?.drizzle
    .update(haexSettings)
    .set({ key: 'theme', value: theme.name })
    .where(eq(haexSettings.key, 'theme'))
  currentTheme.value = theme
}

const onSetVaultNameAsync = async (vaultName: string) => {
  updateVaultNameAsync(vaultName)
}
</script>

<i18n lang="yaml">
de:
  language: Sprache
  design: Design
  vaultName: Vaultname
  save: Änderung speichern

en:
  language: Language
  design: Design
  vaultName: Vault Name
  save: save changes
</i18n>

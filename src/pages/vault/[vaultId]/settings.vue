<template>
  <div
    class="grid grid-rows-2 sm:grid-cols-2 sm:gap-2 p-2 max-w-2xl w-full h-fit"
  >
    <div class="p-2">{{ t('language') }}</div>
    <div><UiDropdownLocale @select="onSelectLocaleAsync" /></div>

    <div class="p-2">{{ t('design') }}</div>
    <div><UiDropdownTheme @select="onSelectThemeAsync" /></div>

    <div class="p-2">{{ t('vaultName.label') }}</div>
    <div>
      <UiInput v-model="currentVaultName" :placeholder="t('vaultName.label')">
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
  await currentVault.value?.drizzle
    .update(haexSettings)
    .set({ key: 'locale', value: locale })
    .where(eq(haexSettings.key, 'locale'))
  await setLocale(locale)
}

const { currentTheme } = storeToRefs(useUiStore())

const onSelectThemeAsync = async (theme: ITheme) => {
  await currentVault.value?.drizzle
    .update(haexSettings)
    .set({ key: 'theme', value: theme.name })
    .where(eq(haexSettings.key, 'theme'))
  currentTheme.value = theme
}

const { add } = useSnackbar()
const onSetVaultNameAsync = async () => {
  try {
    await updateVaultNameAsync(currentVaultName.value)
    add({ text: t('vaultName.update.success'), type: 'success' })
  } catch (error) {
    add({ text: t('vaultName.update.error'), type: 'error' })
  }
}
</script>

<i18n lang="yaml">
de:
  language: Sprache
  design: Design
  save: Änderung speichern
  vaultName:
    label: Vaultname
    update:
      success: Vaultname erfolgreich aktualisiert
      error: Vaultname konnte nicht aktualisiert werden
en:
  language: Language
  design: Design
  save: save changes
  vaultName:
    label: Vault Name
    update:
      success: Vault Name successfully updated
      error: Vault name could not be updated
</i18n>

import * as schema from '@/../src-tauri/database/schemas/vault'
import { eq } from 'drizzle-orm'

export const useVaultSettingsStore = defineStore('vaultSettingsStore', () => {
  const { currentVault, currentVaultName } = storeToRefs(useVaultStore())

  const {
    public: { haexVault },
  } = useRuntimeConfig()

  const syncLocaleAsync = async () => {
    try {
      const app = useNuxtApp()

      const currentLocaleRow = await currentVault.value?.drizzle
        .select()
        .from(schema.haexSettings)
        .where(eq(schema.haexSettings.key, 'locale'))

      if (currentLocaleRow?.[0]?.value) {
        const currentLocale = app.$i18n.availableLocales.find(
          (locale) => locale === currentLocaleRow[0].value,
        )
        await app.$i18n.setLocale(currentLocale ?? app.$i18n.defaultLocale)
      } else {
        await currentVault.value?.drizzle.insert(schema.haexSettings).values({
          id: crypto.randomUUID(),
          key: 'locale',
          value: app.$i18n.locale.value,
        })
      }
    } catch (error) {
      console.log('ERROR syncLocaleAsync', error)
    }
  }

  const syncThemeAsync = async () => {
    const { availableThemes, defaultTheme, currentTheme } = storeToRefs(
      useUiStore(),
    )
    const currentThemeRow = await currentVault.value?.drizzle
      .select()
      .from(schema.haexSettings)
      .where(eq(schema.haexSettings.key, 'theme'))

    if (currentThemeRow?.[0]?.value) {
      const theme = availableThemes.value.find(
        (theme) => theme.name === currentThemeRow[0].value,
      )
      currentTheme.value = theme ?? defaultTheme.value
    } else {
      await currentVault.value?.drizzle.insert(schema.haexSettings).values({
        id: crypto.randomUUID(),
        key: 'theme',
        value: currentTheme.value.value,
      })
    }
  }

  const syncVaultNameAsync = async () => {
    const currentVaultNameRow = await currentVault.value?.drizzle
      .select()
      .from(schema.haexSettings)
      .where(eq(schema.haexSettings.key, 'vaultName'))

    if (currentVaultNameRow?.[0]?.value) {
      currentVaultName.value =
        currentVaultNameRow.at(0)?.value ||
        haexVault.defaultVaultName ||
        'HaexHub'
    } else {
      await currentVault.value?.drizzle.insert(schema.haexSettings).values({
        id: crypto.randomUUID(),
        key: 'vaultName',
        value: currentVaultName.value,
      })
    }
  }

  const updateVaultNameAsync = async (newVaultName?: string | null) => {
    return currentVault.value?.drizzle
      .update(schema.haexSettings)
      .set({ value: newVaultName || haexVault.defaultVaultName || 'HaexHub' })
      .where(eq(schema.haexSettings.key, 'vaultName'))
  }

  return {
    syncLocaleAsync,
    syncThemeAsync,
    syncVaultNameAsync,
    updateVaultNameAsync,
  }
})

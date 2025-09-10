import { and, eq } from 'drizzle-orm'
import { z } from 'zod'
import * as schema from '@/../src-tauri/database/schemas/vault'
import type { Locale } from 'vue-i18n'

export enum VaultSettingsTypeEnum {
  deviceName = 'deviceName',
  settings = 'settings',
}

export enum VaultSettingsKeyEnum {
  locale = 'locale',
  theme = 'theme',
  vaultName = 'vaultName',
}

export const vaultDeviceNameSchema = z.string().min(3).max(255)

export const useVaultSettingsStore = defineStore('vaultSettingsStore', () => {
  const { currentVault, currentVaultName } = storeToRefs(useVaultStore())

  const {
    public: { haexVault },
  } = useRuntimeConfig()

  const syncLocaleAsync = async () => {
    try {
      const app = useNuxtApp()

      const currentLocaleRow =
        await currentVault.value?.drizzle.query.haexSettings.findFirst({
          where: eq(schema.haexSettings.key, VaultSettingsKeyEnum.locale),
        })

      if (currentLocaleRow?.value) {
        const currentLocale = app.$i18n.availableLocales.find(
          (locale) => locale === currentLocaleRow.value,
        )
        await app.$i18n.setLocale(currentLocale ?? app.$i18n.defaultLocale)
      } else {
        await currentVault.value?.drizzle.insert(schema.haexSettings).values({
          id: crypto.randomUUID(),
          key: VaultSettingsKeyEnum.locale,
          type: VaultSettingsTypeEnum.settings,
          value: app.$i18n.locale.value,
        })
      }
    } catch (error) {
      console.log('ERROR syncLocaleAsync', error)
    }
  }

  const updateLocaleAsync = async (locale: Locale) => {
    await currentVault.value?.drizzle
      .update(schema.haexSettings)
      .set({ key: VaultSettingsKeyEnum.locale, value: locale })
      .where(
        and(
          eq(schema.haexSettings.key, VaultSettingsKeyEnum.locale),
          eq(schema.haexSettings.type, VaultSettingsTypeEnum.settings),
        ),
      )
  }
  const syncThemeAsync = async () => {
    const { defaultTheme, currentTheme, currentThemeName, availableThemes } =
      storeToRefs(useUiStore())

    const currentThemeRow =
      await currentVault.value?.drizzle.query.haexSettings.findFirst({
        where: eq(schema.haexSettings.key, VaultSettingsKeyEnum.theme),
      })

    if (currentThemeRow?.value) {
      const theme = availableThemes.value.find(
        (theme) => theme.value === currentThemeRow.value,
      )
      currentThemeName.value = theme?.value || defaultTheme.value
    } else {
      await currentVault.value?.drizzle.insert(schema.haexSettings).values({
        id: crypto.randomUUID(),
        key: VaultSettingsKeyEnum.theme,
        type: VaultSettingsTypeEnum.settings,
        value: currentTheme.value?.value,
      })
    }
  }

  const updateThemeAsync = async (theme: string) => {
    return await currentVault.value?.drizzle
      .update(schema.haexSettings)
      .set({ key: VaultSettingsKeyEnum.theme, value: theme })
      .where(eq(schema.haexSettings.key, VaultSettingsKeyEnum.theme))
  }

  const syncVaultNameAsync = async () => {
    const currentVaultNameRow =
      await currentVault.value?.drizzle.query.haexSettings.findFirst({
        where: eq(schema.haexSettings.key, VaultSettingsKeyEnum.vaultName),
      })

    if (currentVaultNameRow?.value) {
      currentVaultName.value =
        currentVaultNameRow.value || haexVault.defaultVaultName || 'HaexHub'
    } else {
      await currentVault.value?.drizzle.insert(schema.haexSettings).values({
        id: crypto.randomUUID(),
        key: VaultSettingsKeyEnum.vaultName,
        type: VaultSettingsTypeEnum.settings,
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

  const readDeviceNameAsync = async (id: string) => {
    const { currentVault } = useVaultStore()

    const deviceName =
      await currentVault?.drizzle?.query.haexSettings.findFirst({
        where: and(
          eq(schema.haexSettings.type, VaultSettingsTypeEnum.deviceName),
          eq(schema.haexSettings.key, id),
        ),
      })
    console.log('readDeviceNameAsync', deviceName)
    return deviceName
  }

  const addDeviceNameAsync = async ({
    deviceId,
    deviceName,
  }: {
    deviceId: string
    deviceName: string
  }) => {
    const { currentVault } = useVaultStore()

    const isNameOk = vaultDeviceNameSchema.safeParse(deviceName)
    if (!isNameOk.success) {
      console.log('deviceName not OK', isNameOk.error)
      return
    }

    return currentVault?.drizzle?.insert(schema.haexSettings).values({
      id: crypto.randomUUID(),
      type: VaultSettingsTypeEnum.deviceName,
      key: deviceId,
      value: deviceName,
    })
  }

  const updateDeviceNameAsync = async ({
    deviceId,
    deviceName,
  }: {
    deviceId: string
    deviceName: string
  }) => {
    const { currentVault } = useVaultStore()

    const isNameOk = vaultDeviceNameSchema.safeParse(deviceName)
    if (!isNameOk.success) return

    return currentVault?.drizzle
      ?.update(schema.haexSettings)
      .set({
        value: deviceName,
      })
      .where(
        and(
          eq(schema.haexSettings.key, deviceId),
          eq(schema.haexSettings.type, VaultSettingsTypeEnum.deviceName),
        ),
      )
  }

  return {
    addDeviceNameAsync,
    readDeviceNameAsync,
    syncLocaleAsync,
    syncThemeAsync,
    syncVaultNameAsync,
    updateDeviceNameAsync,
    updateLocaleAsync,
    updateThemeAsync,
    updateVaultNameAsync,
  }
})

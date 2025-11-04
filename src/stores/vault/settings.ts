import { and, eq } from 'drizzle-orm'
import { z } from 'zod'
import * as schema from '~/database/schemas/haex'
import type { Locale } from 'vue-i18n'

export enum VaultSettingsTypeEnum {
  settings = 'settings',
  system = 'system',
}

export enum VaultSettingsKeyEnum {
  locale = 'locale',
  theme = 'theme',
  vaultName = 'vaultName',
  desktopIconSize = 'desktopIconSize',
}

export enum DesktopIconSizePreset {
  small = 'small',
  medium = 'medium',
  large = 'large',
  extraLarge = 'extra-large',
}

export const iconSizePresetValues: Record<DesktopIconSizePreset, number> = {
  [DesktopIconSizePreset.small]: 60,
  [DesktopIconSizePreset.medium]: 80,
  [DesktopIconSizePreset.large]: 120,
  [DesktopIconSizePreset.extraLarge]: 160,
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

  const readDeviceNameAsync = async (deviceId?: string) => {
    const { currentVault } = useVaultStore()

    if (!deviceId) return undefined

    const device =
      await currentVault?.drizzle?.query.haexDevices.findFirst({
        where: eq(schema.haexDevices.deviceId, deviceId),
      })

    // Workaround für Drizzle Bug: findFirst gibt manchmal Objekt mit undefined Werten zurück
    // https://github.com/drizzle-team/drizzle-orm/issues/3872
    // Prüfe ob das Device wirklich existiert (id muss gesetzt sein, da NOT NULL)
    if (!device?.id) return undefined

    return device
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

    return currentVault?.drizzle?.insert(schema.haexDevices).values({
      deviceId,
      name: deviceName,
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
      ?.update(schema.haexDevices)
      .set({
        name: deviceName,
      })
      .where(eq(schema.haexDevices.deviceId, deviceId))
  }

  const syncDesktopIconSizeAsync = async (deviceInternalId: string) => {
    const iconSizeRow =
      await currentVault.value?.drizzle.query.haexSettings.findFirst({
        where: and(
          eq(schema.haexSettings.deviceId, deviceInternalId),
          eq(schema.haexSettings.key, VaultSettingsKeyEnum.desktopIconSize),
          eq(schema.haexSettings.type, VaultSettingsTypeEnum.system),
        ),
      })

    if (!iconSizeRow?.id) {
      // Kein Eintrag vorhanden, erstelle einen mit Default (medium)
      await currentVault.value?.drizzle.insert(schema.haexSettings).values({
        deviceId: deviceInternalId,
        key: VaultSettingsKeyEnum.desktopIconSize,
        type: VaultSettingsTypeEnum.system,
        value: DesktopIconSizePreset.medium,
      })
      return DesktopIconSizePreset.medium
    }

    return iconSizeRow.value as DesktopIconSizePreset
  }

  const updateDesktopIconSizeAsync = async (
    deviceInternalId: string,
    preset: DesktopIconSizePreset,
  ) => {
    return await currentVault.value?.drizzle
      .update(schema.haexSettings)
      .set({ value: preset })
      .where(
        and(
          eq(schema.haexSettings.deviceId, deviceInternalId),
          eq(schema.haexSettings.key, VaultSettingsKeyEnum.desktopIconSize),
          eq(schema.haexSettings.type, VaultSettingsTypeEnum.system),
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
    syncDesktopIconSizeAsync,
    updateDesktopIconSizeAsync,
  }
})

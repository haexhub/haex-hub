<template>
  <div class="w-full h-full bg-default">
    <div class="grid grid-cols-2 p-2">
      <div class="p-2">{{ t('language') }}</div>
      <div><UiDropdownLocale @select="onSelectLocaleAsync" /></div>

      <div class="p-2">{{ t('design') }}</div>
      <div><UiDropdownTheme @select="onSelectThemeAsync" /></div>

      <div class="p-2">{{ t('vaultName.label') }}</div>
      <div>
        <UiInput
          v-model="currentVaultName"
          :placeholder="t('vaultName.label')"
          @change="onSetVaultNameAsync"
        />
      </div>

      <div class="p-2">{{ t('notifications.label') }}</div>
      <div>
        <UiButton
          :label="t('notifications.requestPermission')"
          @click="requestNotificationPermissionAsync"
        />
      </div>

      <div class="p-2">{{ t('deviceName.label') }}</div>
      <div>
        <UiInput
          v-model="deviceName"
          :placeholder="t('deviceName.label')"
          @change="onUpdateDeviceNameAsync"
        />
      </div>

      <div class="h-full"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Locale } from 'vue-i18n'

const { t, setLocale } = useI18n()

const { currentVaultName } = storeToRefs(useVaultStore())
const { updateVaultNameAsync, updateLocaleAsync, updateThemeAsync } =
  useVaultSettingsStore()

const onSelectLocaleAsync = async (locale: Locale) => {
  await updateLocaleAsync(locale)
  await setLocale(locale)
}

const { currentThemeName } = storeToRefs(useUiStore())

const onSelectThemeAsync = async (theme: string) => {
  currentThemeName.value = theme
  console.log('onSelectThemeAsync', currentThemeName.value)
  await updateThemeAsync(theme)
}

const { add } = useToast()

const onSetVaultNameAsync = async () => {
  try {
    await updateVaultNameAsync(currentVaultName.value)
    add({ description: t('vaultName.update.success'), color: 'success' })
  } catch (error) {
    console.error(error)
    add({ description: t('vaultName.update.error'), color: 'error' })
  }
}

const { requestNotificationPermissionAsync } = useNotificationStore()

const { deviceName } = storeToRefs(useDeviceStore())
const { updateDeviceNameAsync, readDeviceNameAsync } = useDeviceStore()

onMounted(async () => {
  await readDeviceNameAsync()
})

const onUpdateDeviceNameAsync = async () => {
  const check = vaultDeviceNameSchema.safeParse(deviceName.value)
  if (!check.success) return
  try {
    await updateDeviceNameAsync({ name: deviceName.value })
    add({ description: t('deviceName.update.success'), color: 'success' })
  } catch (error) {
    console.log(error)
    add({ description: t('deviceName.update.error'), color: 'error' })
  }
}
</script>

<i18n lang="yaml">
de:
  language: Sprache
  design: Design
  save: Änderung speichern
  notifications:
    label: Benachrichtigungen
    requestPermission: Benachrichtigung erlauben
  vaultName:
    label: Vaultname
    update:
      success: Vaultname erfolgreich aktualisiert
      error: Vaultname konnte nicht aktualisiert werden
  deviceName:
    label: Gerätename
    update:
      success: Gerätename wurde erfolgreich aktualisiert
      error: Gerätename konnte nich aktualisiert werden
en:
  language: Language
  design: Design
  save: save changes
  notifications:
    label: Notifications
    requestPermission: Grant Permission
  vaultName:
    label: Vault Name
    update:
      success: Vault Name successfully updated
      error: Vault name could not be updated
  deviceName:
    label: Device name
    update:
      success: Device name has been successfully updated
      error: Device name could not be updated
</i18n>

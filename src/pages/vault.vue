<template>
  <div class="h-full">
    <NuxtLayout name="app">
      <NuxtPage />
    </NuxtLayout>

    <div class="hidden">
      <UiDialogConfirm
        :confirm-label="t('newDevice.save')"
        :title="t('newDevice.title')"
        @abort="showNewDeviceDialog = false"
        @confirm="onSetDeviceNameAsync"
        confirm-icon="mdi:content-save-outline"
        v-model:open="showNewDeviceDialog"
      >
        <div class="flex flex-col gap-4">
          <p>{{ t('newDevice.intro') }}</p>
          <p>
            {{ t('newDevice.setName') }}
          </p>
          {{ deviceId }}
          <UiInput
            v-model="newDeviceName"
            :label="t('newDevice.label')"
            :rules="vaultDeviceNameSchema"
          />
        </div>
      </UiDialogConfirm>
    </div>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  middleware: 'database',
})

const { t } = useI18n()

const showNewDeviceDialog = ref(false)

const { hostname } = storeToRefs(useDeviceStore())

const newDeviceName = ref<string>('unknown')

const { readNotificationsAsync } = useNotificationStore()
const { isKnownDeviceAsync } = useDeviceStore()
const { loadExtensionsAsync } = useExtensionsStore()
const { setDeviceIdIfNotExistsAsync, addDeviceNameAsync } = useDeviceStore()
const { deviceId } = storeToRefs(useDeviceStore())

onMounted(async () => {
  await setDeviceIdIfNotExistsAsync()
  await loadExtensionsAsync()
  await readNotificationsAsync()

  if (!(await isKnownDeviceAsync())) {
    console.log('not known device')
    newDeviceName.value = hostname.value ?? 'unknown'
    showNewDeviceDialog.value = true
  }
})

const { add } = useSnackbar()
const onSetDeviceNameAsync = async () => {
  try {
    const check = vaultDeviceNameSchema.safeParse(newDeviceName.value)
    if (!check.success) {
      console.log('check failed', check.error)
      return
    }

    await addDeviceNameAsync({ name: newDeviceName.value })
    showNewDeviceDialog.value = false
    add({ type: 'success', text: t('newDevice.success') })
  } catch (error) {
    add({ type: 'error', text: t('newDevice.error') })
  }
}
</script>

<i18n lang="yaml">
de:
  newDevice:
    title: Neues Gerät erkannt
    save: Speichern
    label: Name
    intro: Offenbar öffnest du das erste Mal diese Vault auf diesem Gerät.
    setName: Bitte gib diesem Gerät einen für dich sprechenden Namen. Dadurch kannst du später besser nachverfolgen, welche Änderungen von welchem Gerät erfolgt sind.
    success: Name erfolgreich gespeichert
    error: Name konnt nicht gespeichert werden

en:
  newDevice:
    title: New device recognized
    save: Save
    label: Name
    intro: This is obviously your first time with this Vault on this device.
    setName: Please give this device a name that is meaningful to you. This will make it easier for you to track which changes have been made by which device.
    success: Name successfully saved
    error: Name could not be saved
</i18n>

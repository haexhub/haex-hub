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

      <div class="p-2">{{ t('workspaceBackground.label') }}</div>
      <div class="flex gap-2">
        <UiButton
          :label="t('workspaceBackground.choose')"
          @click="selectBackgroundImage"
        />
        <UiButton
          v-if="currentWorkspace?.background"
          :label="t('workspaceBackground.remove.label')"
          color="error"
          @click="removeBackgroundImage"
        />
      </div>

      <div class="h-full" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Locale } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import {
  readFile,
  writeFile,
  mkdir,
  exists,
  remove,
} from '@tauri-apps/plugin-fs'
import { appLocalDataDir } from '@tauri-apps/api/path'

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

const workspaceStore = useWorkspaceStore()
const { currentWorkspace } = storeToRefs(workspaceStore)
const { updateWorkspaceBackgroundAsync } = workspaceStore

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

const selectBackgroundImage = async () => {
  if (!currentWorkspace.value) return

  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Images',
          extensions: ['png', 'jpg', 'jpeg', 'webp'],
        },
      ],
    })

    if (!selected || typeof selected !== 'string') {
      return
    }

    // Read the selected file (works with Android photo picker URIs)
    let fileData: Uint8Array
    try {
      fileData = await readFile(selected)
    } catch (readError) {
      add({
        description: `Fehler beim Lesen: ${readError instanceof Error ? readError.message : String(readError)}`,
        color: 'error',
      })
      return
    }

    // Detect file type from file signature
    let ext = 'jpg' // default
    if (fileData.length > 4) {
      // PNG signature: 89 50 4E 47
      if (
        fileData[0] === 0x89 &&
        fileData[1] === 0x50 &&
        fileData[2] === 0x4e &&
        fileData[3] === 0x47
      ) {
        ext = 'png'
      }
      // JPEG signature: FF D8 FF
      else if (
        fileData[0] === 0xff &&
        fileData[1] === 0xd8 &&
        fileData[2] === 0xff
      ) {
        ext = 'jpg'
      }
      // WebP signature: RIFF xxxx WEBP
      else if (
        fileData[0] === 0x52 &&
        fileData[1] === 0x49 &&
        fileData[2] === 0x46 &&
        fileData[3] === 0x46
      ) {
        ext = 'webp'
      }
    }

    // Get app local data directory
    const appDataPath = await appLocalDataDir()

    // Construct target path manually to avoid path joining issues
    const fileName = `workspace-${currentWorkspace.value.id}-background.${ext}`
    const targetPath = `${appDataPath}/files/${fileName}`

    // Create parent directory if it doesn't exist
    const parentDir = `${appDataPath}/files`
    try {
      if (!(await exists(parentDir))) {
        await mkdir(parentDir, { recursive: true })
      }
    } catch (mkdirError) {
      add({
        description: `Fehler beim Erstellen des Ordners: ${mkdirError instanceof Error ? mkdirError.message : String(mkdirError)}`,
        color: 'error',
      })
      return
    }

    // Write file to app data directory
    try {
      await writeFile(targetPath, fileData)
    } catch (writeError) {
      add({
        description: `Fehler beim Schreiben: ${writeError instanceof Error ? writeError.message : String(writeError)}`,
        color: 'error',
      })
      return
    }

    // Store the absolute file path in database
    try {
      await updateWorkspaceBackgroundAsync(
        currentWorkspace.value.id,
        targetPath,
      )
      add({
        description: t('workspaceBackground.update.success'),
        color: 'success',
      })
    } catch (dbError) {
      add({
        description: `Fehler beim DB-Update: ${dbError instanceof Error ? dbError.message : String(dbError)}`,
        color: 'error',
      })
    }
  } catch (error) {
    console.error('Error selecting background:', error)
    add({
      description: `${t('workspaceBackground.update.error')}: ${error instanceof Error ? error.message : String(error)}`,
      color: 'error',
    })
  }
}

const removeBackgroundImage = async () => {
  if (!currentWorkspace.value) return

  try {
    // Delete the background file if it exists
    if (currentWorkspace.value.background) {
      try {
        // The background field contains the absolute file path
        if (await exists(currentWorkspace.value.background)) {
          await remove(currentWorkspace.value.background)
        }
      } catch (err) {
        console.warn('Could not delete background file:', err)
        // Continue anyway to clear the database entry
      }
    }

    await updateWorkspaceBackgroundAsync(currentWorkspace.value.id, null)
    add({
      description: t('workspaceBackground.remove.success'),
      color: 'success',
    })
  } catch (error) {
    console.error('Error removing background:', error)
    add({ description: t('workspaceBackground.remove.error'), color: 'error' })
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
  workspaceBackground:
    label: Workspace-Hintergrund
    choose: Bild auswählen
    update:
      success: Hintergrund erfolgreich aktualisiert
      error: Fehler beim Aktualisieren des Hintergrunds
    remove:
      label: Hintergrund entfernen
      success: Hintergrund erfolgreich entfernt
      error: Fehler beim Entfernen des Hintergrunds
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
  workspaceBackground:
    label: Workspace Background
    choose: Choose Image
    update:
      success: Background successfully updated
      error: Error updating background
    remove:
      label: Remove Background
      success: Background successfully removed
      error: Error removing background
</i18n>

import { invoke } from '@tauri-apps/api/core'
import { readFile } from '@tauri-apps/plugin-fs'
import { getExtensionUrl } from '~/utils/extension'

import type {
  IHaexHubExtension,
  IHaexHubExtensionManifest,
} from '~/types/haexhub'
import type { ExtensionPreview } from '@bindings/ExtensionPreview'
import type { ExtensionPermissions } from '~~/src-tauri/bindings/ExtensionPermissions'
import type { ExtensionInfoResponse } from '~~/src-tauri/bindings/ExtensionInfoResponse'

/* const manifestFileName = 'manifest.json'
const logoFileName = 'icon.svg' */

export const useExtensionsStore = defineStore('extensionsStore', () => {
  const availableExtensions = ref<IHaexHubExtension[]>([])
  const currentRoute = useRouter().currentRoute

  const currentExtensionId = computed(() =>
    getSingleRouteParam(currentRoute.value.params.extensionId),
  )

  const currentExtension = computed(() => {
    if (!currentExtensionId.value) return null

    return (
      availableExtensions.value.find(
        (ext) => ext.id === currentExtensionId.value,
      ) ?? null
    )
  })

  /* const { addNotificationAsync } = useNotificationStore() */

  /*  const extensionLinks = computed<ISidebarItem[]>(() =>
    availableExtensions.value
      .filter((extension) => extension.enabled && extension.installed)
      .map((extension) => ({
        icon: extension.icon ?? '',
        id: extension.id,
        name: extension.name ?? '',
        tooltip: extension.name ?? '',
        to: { name: 'haexExtension', params: { extensionId: extension.id } },
      })),
  ) */

  const isActive = (id: string) =>
    computed(
      () =>
        currentRoute.value.name === 'extension' &&
        currentRoute.value.params.extensionId === id,
    )

  const extensionEntry = computed(() => {
    if (
      !currentExtension.value?.version ||
      !currentExtension.value?.publicKey ||
      !currentExtension.value?.name
    )
      return null

    return getExtensionUrl(
      currentExtension.value.publicKey,
      currentExtension.value.name,
      currentExtension.value.version,
      'index.html',
      currentExtension.value.devServerUrl ?? undefined
    )
  })

  /* const getExtensionPathAsync = async (
    extensionId?: string,
    version?: string,
  ) => {
    if (!extensionId || !version) return ''
    return await join(await appDataDir(), 'extensions', extensionId, version)
  } */

  /* const checkSourceExtensionDirectoryAsync = async (
    extensionDirectory: string,
  ) => {
    try {
      const dir = await readDir(extensionDirectory)
      const manifest = dir.find(
        (entry) => entry.name === manifestFileName && entry.isFile,
      )
      if (!manifest) throw new Error('Kein Manifest für Erweiterung gefunden')

      const logo = dir.find((item) => item.isFile && item.name === logoFileName)
      if (!logo) throw new Error('Logo fehlt')
      console.log('found icon', logo)

      return true
    } catch (error) {
      console.error(error)
      addNotificationAsync({ type: 'error', text: JSON.stringify(error) })
      //throw error //new Error(`Keine Leseberechtigung für Ordner ${extensionDirectory}`);
    }
  } */

  const loadExtensionsAsync = async () => {
    try {
      const extensions =
        await invoke<ExtensionInfoResponse[]>('get_all_extensions')

      // ExtensionInfoResponse is now directly compatible with IHaexHubExtension
      availableExtensions.value = extensions
    } catch (error) {
      console.error('Fehler beim Laden der Extensions:', error)
      throw error
    }
  }

  /* const loadExtensionsAsync = async () => {
    const { currentVault } = storeToRefs(useVaultStore())

    const extensions =
      (await currentVault.value?.drizzle.select().from(haexExtensions)) ?? []

    //if (!extensions?.length) return false;

    const installedExtensions = await filterAsync(
      extensions,
      isExtensionInstalledAsync,
    )
    console.log('loadExtensionsAsync installedExtensions', installedExtensions)

    availableExtensions.value =
      extensions.map((extension) => ({
        id: extension.id,
        name: extension.name ?? '',
        icon: extension.icon ?? '',
        author: extension.author ?? '',
        version: extension.version ?? '',
        enabled: extension.enabled ? true : false,
        installed: installedExtensions.includes(extension),
      })) ?? []

    console.log('loadExtensionsAsync', availableExtensions.value)
    return true
  } */

  const installAsync = async (
    sourcePath: string | null,
    permissions?: ExtensionPermissions,
  ) => {
    if (!sourcePath) throw new Error('Kein Pfad angegeben')

    try {
      // Read file as bytes (works with content URIs on Android)
      const fileBytes = await readFile(sourcePath)

      const extensionId = await invoke<string>(
        'install_extension_with_permissions',
        {
          fileBytes: Array.from(fileBytes),
          customPermissions: permissions,
        },
      )
      return extensionId
    } catch (error) {
      console.error('Fehler bei Extension-Installation:', error)
      throw error
    }
  }

  const removeExtensionAsync = async (
    publicKey: string,
    name: string,
    version: string,
  ) => {
    try {
      await invoke('remove_extension', {
        publicKey,
        name,
        version,
      })
    } catch (error) {
      console.error('Fehler beim Entfernen der Extension:', error)
      throw error
    }
  }

  /* const removeExtensionAsync = async (id: string, version: string) => {
    try {
      console.log('remove extension', id, version)
      await removeExtensionFromVaultAsync(id, version)
      await removeExtensionFilesAsync(id, version)
    } catch (error) {
      throw new Error(JSON.stringify(error))
    }
  } */

  const isExtensionInstalledAsync = async ({
    publicKey,
    name,
    version,
  }: {
    publicKey: string
    name: string
    version: string
  }) => {
    try {
      return await invoke<boolean>('is_extension_installed', {
        publicKey,
        name,
        extensionVersion: version,
      })
    } catch (error) {
      console.error('Fehler beim Prüfen der Extension:', error)
      return false
    }
  }

  const checkManifest = (
    manifestFile: unknown,
  ): manifestFile is IHaexHubExtensionManifest => {
    const errors = []

    if (typeof manifestFile !== 'object' || manifestFile === null) {
      errors.push('Manifest ist falsch')
      return false
    }

    if (!('id' in manifestFile) || typeof manifestFile.id !== 'string')
      errors.push('Keine ID vergeben')

    if (!('name' in manifestFile) || typeof manifestFile.name !== 'string')
      errors.push('Name fehlt')

    if (!('entry' in manifestFile) || typeof manifestFile.entry !== 'string')
      errors.push('Entry fehlerhaft')

    if (!('author' in manifestFile) || typeof manifestFile.author !== 'string')
      errors.push('Author fehlt')

    if (!('url' in manifestFile) || typeof manifestFile.url !== 'string')
      errors.push('Url fehlt')

    if (
      !('version' in manifestFile) ||
      typeof manifestFile.version !== 'string'
    )
      errors.push('Version fehlt')

    if (
      !('permissions' in manifestFile) ||
      typeof manifestFile.permissions !== 'object' ||
      manifestFile.permissions === null
    ) {
      errors.push('Berechtigungen fehlen')
    }

    if (errors.length) throw errors

    /* const permissions = manifestFile.permissions as Partial<IHaexHubExtensionManifest["permissions"]>;
    if (
      ("database" in permissions &&
        (typeof permissions.database !== "object" || permissions.database === null)) ||
      ("filesystem" in permissions && typeof permissions.filesystem !== "object") ||
      permissions.filesystem === null
    ) {
      return false;
    } */

    return true
  }

  const preview = ref<ExtensionPreview>()

  const previewManifestAsync = async (extensionPath: string) => {
    // Read file as bytes (works with content URIs on Android)
    const fileBytes = await readFile(extensionPath)

    preview.value = await invoke<ExtensionPreview>('preview_extension', {
      fileBytes: Array.from(fileBytes),
    })
    return preview.value
  }

  return {
    availableExtensions,
    checkManifest,
    currentExtension,
    currentExtensionId,
    extensionEntry,
    installAsync,
    isActive,
    isExtensionInstalledAsync,
    loadExtensionsAsync,
    previewManifestAsync,
    removeExtensionAsync,
  }
})

/* const getMimeType = (file: string) => {
  if (file.endsWith('.css')) return 'text/css'
  if (file.endsWith('.js')) return 'text/javascript'
  return 'text/plain'
} */

/* const removeExtensionFromVaultAsync = async (
  id: string | null,
  version: string | null,
) => {
  if (!id)
    throw new Error(
      'Erweiterung kann nicht gelöscht werden. Es keine ID angegeben',
    )

  if (!version)
    throw new Error(
      'Erweiterung kann nicht gelöscht werden. Es wurde keine Version angegeben',
    )

  const { currentVault } = useVaultStore()
  const removedExtensions = await currentVault?.drizzle
    .delete(haexExtensions)
    .where(and(eq(haexExtensions.id, id), eq(haexExtensions.version, version)))
  return removedExtensions
} */

/* const removeExtensionFilesAsync = async (
  id: string | null,
  version: string | null,
) => {
  try {
    const { getExtensionPathAsync } = useExtensionsStore()
    if (!id)
      throw new Error(
        'Erweiterung kann nicht gelöscht werden. Es keine ID angegeben',
      )

    if (!version)
      throw new Error(
        'Erweiterung kann nicht gelöscht werden. Es wurde keine Version angegeben',
      )

    const extensionDirectory = await getExtensionPathAsync(id, version)
    await remove(extensionDirectory, {
      recursive: true,
    })
  } catch (error) {
    console.error('ERROR removeExtensionFilesAsync', error)
    throw new Error(JSON.stringify(error))
  }
} */

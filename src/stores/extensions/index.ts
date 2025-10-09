import { invoke } from '@tauri-apps/api/core'
import { readFile } from '@tauri-apps/plugin-fs'
import { EXTENSION_PROTOCOL_PREFIX } from '~/config/constants'

import type {
  IHaexHubExtension,
  IHaexHubExtensionManifest,
} from '~/types/haexhub'
import type { ExtensionPreview } from '@bindings/ExtensionPreview'
import type { ExtensionPermissions } from '~~/src-tauri/bindings/ExtensionPermissions'

interface ExtensionInfoResponse {
  keyHash: string
  name: string
  fullId: string
  version: string
  displayName: string | null
  namespace: string | null
  allowedOrigin: string
}

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
      !currentExtension.value?.id ||
      !currentExtension.value?.name
    )
      return null

    // Extract key_hash from full_extension_id (everything before first underscore)
    const firstUnderscoreIndex = currentExtension.value.id.indexOf('_')
    if (firstUnderscoreIndex === -1) {
      console.error(
        'Invalid full_extension_id format:',
        currentExtension.value.id,
      )
      return null
    }

    const keyHash = currentExtension.value.id.substring(0, firstUnderscoreIndex)

    const encodedInfo = encodeExtensionInfo(
      keyHash,
      currentExtension.value.name,
      currentExtension.value.version,
    )

    return `${EXTENSION_PROTOCOL_PREFIX}localhost/${encodedInfo}/index.html`
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

      availableExtensions.value = extensions.map((ext) => ({
        id: ext.fullId,
        name: ext.displayName || ext.name,
        version: ext.version,
        author: ext.namespace,
        icon: null,
        enabled: true,
      }))
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

  /* const installAsync = async (extensionDirectory: string | null) => {
    try {
      if (!extensionDirectory)
        throw new Error('Kein Ordner für Erweiterung angegeben')
      const manifestPath = await join(extensionDirectory, manifestFileName)
      const manifest = (await JSON.parse(
        await readTextFile(manifestPath),
      )) as IHaexHubExtensionManifest

      const destination = await getExtensionPathAsync(
        manifest.id,
        manifest.version,
      )

      await checkSourceExtensionDirectoryAsync(extensionDirectory)

      await invoke('copy_directory', {
        source: extensionDirectory,
        destination,
      })

      const logoFilePath = await join(destination, logoFileName)
      const logo = await readTextFile(logoFilePath)

      const { currentVault } = storeToRefs(useVaultStore())
      const res = await currentVault.value?.drizzle
        .insert(haexExtensions)
        .values({
          id: manifest.id,
          name: manifest.name,
          author: manifest.author,
          enabled: true,
          url: manifest.url,
          version: manifest.version,
          icon: logo,
        })

      console.log('insert extensions', res)
      addNotificationAsync({
        type: 'success',
        text: `${manifest.name} wurde installiert`,
      })
    } catch (error) {
      addNotificationAsync({ type: 'error', text: JSON.stringify(error) })
      throw error
    }
  } */

  const removeExtensionAsync = async (extensionId: string, version: string) => {
    try {
      await invoke('remove_extension', {
        extensionId,
        extensionVersion: version,
      })
    } catch (error) {
      console.error('Fehler beim Entfernen der Extension:', error)
      throw error
    }
  }

  const removeExtensionByFullIdAsync = async (fullExtensionId: string) => {
    try {
      await invoke('remove_extension_by_full_id', {
        fullExtensionId,
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
    id,
    version,
  }: {
    id: string
    version: string
  }) => {
    try {
      return await invoke<boolean>('is_extension_installed', {
        extensionId: id,
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
  /* const readManifestFileAsync = async (
    extensionId: string,
    version: string,
  ) => {
    try {
      if (!(await isExtensionInstalledAsync({ id: extensionId, version })))
        return null

      const extensionPath = await getExtensionPathAsync(
        extensionId,
        `${version}`,
      )
      const manifestPath = await join(extensionPath, manifestFileName)
      const manifest = (await JSON.parse(
        await readTextFile(manifestPath),
      )) as IHaexHubExtensionManifest

      return manifest
    } catch (error) {
      addNotificationAsync({ type: 'error', text: JSON.stringify(error) })
      console.error('ERROR readManifestFileAsync', error)
    }
  } */

  /* const extensionEntry = computedAsync(
    async () => {
      try {


        if (!currentExtension.value?.id || !currentExtension.value.version) {
          console.log('extension id or entry missing', currentExtension.value)
          return '' // "no mani: " + currentExtension.value;
        }

        const extensionPath = await getExtensionPathAsync(
          currentExtension.value?.id,
          currentExtension.value?.version,
        ) //await join(await resourceDir(), currentExtension.value.. extensionDir, entryFileName);

        console.log('extensionEntry extensionPath', extensionPath)
        const manifest = await readManifestFileAsync(
          currentExtension.value.id,
          currentExtension.value.version,
        )

        if (!manifest) return '' //"no manifest readable";

        //const entryPath = await join(extensionPath, manifest.entry)

        const hexName = stringToHex(
          JSON.stringify({
            id: currentExtension.value.id,
            version: currentExtension.value.version,
          }),
        )

        return `haex-extension://${hexName}`
      } catch (error) {
        console.error('ERROR extensionEntry', error)
      }
    },
    null,
    { lazy: true },
  ) */

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
    removeExtensionByFullIdAsync,
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

function encodeExtensionInfo(
  keyHash: string,
  name: string,
  version: string,
): string {
  const info = {
    key_hash: keyHash,
    name: name,
    version: version,
  }
  const jsonString = JSON.stringify(info)
  const bytes = new TextEncoder().encode(jsonString)
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
}

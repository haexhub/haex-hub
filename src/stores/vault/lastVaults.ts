import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'

/* interface ILastVault {
  lastUsed: Date
  name: string
  path: string
} */

export interface IVaultInfo {
  name: string
  path: string
  lastAccess: Date
}

export const useLastVaultStore = defineStore('lastVaultStore', () => {
  const {
    public: { haexVault },
  } = useRuntimeConfig()

  const lastVaults = ref<IVaultInfo[]>([])

  const keyName = 'lastVaults'

  const getStoreAsync = async () => {
    return await load(haexVault.lastVaultFileName || 'lastVaults.json')
  }

  const syncLastVaultsAsync = async () => {
    lastVaults.value =
      (await listVaultsAsync()).sort(
        (a, b) => +new Date(b.lastAccess) - +new Date(a.lastAccess),
      ) ?? []

    return lastVaults.value
  }

  const listVaultsAsync = async () => {
    lastVaults.value = await invoke<IVaultInfo[]>('list_vaults')
    return lastVaults.value
  }

  const addVaultAsync = async ({
    name,
    path,
  }: {
    name?: string
    path: string
  }) => {
    if (!lastVaults.value) await syncLastVaultsAsync()

    const saveName = name || getFileNameFromPath(path)
    lastVaults.value = lastVaults.value.filter((vault) => vault.path !== path)
    lastVaults.value.push({ lastAccess: new Date(), name: saveName, path })
    await saveLastVaultsAsync()
  }

  const removeVaultAsync = async (vaultPath: string) => {
    lastVaults.value = lastVaults.value.filter(
      (vault) => vault.path !== vaultPath,
    )
    await saveLastVaultsAsync()
  }

  const saveLastVaultsAsync = async () => {
    const store = await getStoreAsync()
    await store.set(keyName, lastVaults.value)
    await syncLastVaultsAsync()
  }

  return {
    addVaultAsync,
    syncLastVaultsAsync,
    lastVaults,
    removeVaultAsync,
    saveLastVaultsAsync,
  }
})

const getFileNameFromPath = (path: string) => {
  const lastBackslashIndex = path.lastIndexOf('\\')
  const lastSlashIndex = path.lastIndexOf('/')

  const lastIndex = Math.max(lastBackslashIndex, lastSlashIndex)

  const fileName = path.substring(lastIndex + 1)

  return fileName
}

import { invoke } from '@tauri-apps/api/core'
import type { VaultInfo } from '@bindings/VaultInfo'

export const useLastVaultStore = defineStore('lastVaultStore', () => {
  const lastVaults = ref<VaultInfo[]>([])

  const syncLastVaultsAsync = async () => {
    lastVaults.value =
      (await listVaultsAsync()).sort(
        (a, b) => +new Date(`${b.lastAccess}`) - +new Date(`${a.lastAccess}`),
      ) ?? []

    return lastVaults.value
  }

  const listVaultsAsync = async () => {
    lastVaults.value = await invoke<VaultInfo[]>('list_vaults')
    return lastVaults.value
  }

  const removeVaultAsync = async (vaultName: string) => {
    return await invoke('delete_vault', { vaultName })
  }

  return {
    syncLastVaultsAsync,
    lastVaults,
    removeVaultAsync,
  }
})

import { load } from '@tauri-apps/plugin-store'

export const useVaultInstanceStore = defineStore('vaultInstanceStore', () => {
  const instanceId = ref<string>()

  const getInstanceIdAsync = async () => {
    const store = await getStoreAsync()
    instanceId.value = await store.get<string>('id')

    return instanceId.value
  }

  const getStoreAsync = async () => {
    const {
      public: { haexVault },
    } = useRuntimeConfig()

    return await load(haexVault.instanceFileName || 'instance.json')
  }

  const setInstanceIdAsync = async (id?: string) => {
    const store = await getStoreAsync()
    const _id = id || crypto.randomUUID()
    await store.set('id', _id)

    return _id
  }

  const setInstanceIdIfNotExistsAsync = async () => {
    const id = await getInstanceIdAsync()
    return id ?? (await setInstanceIdAsync())
  }

  const isFirstTimeAsync = async () => {
    const { currentVault } = useVaultStore()

    currentVault.drizzle.select
    return !(await getInstanceIdAsync())
  }

  return {
    instanceId,
    isFirstTimeAsync,
    setInstanceIdAsync,
    setInstanceIdIfNotExistsAsync,
  }
})

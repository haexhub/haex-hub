import { load } from '@tauri-apps/plugin-store'
import {
  hostname as tauriHostname,
  platform as tauriPlatform,
} from '@tauri-apps/plugin-os'

export const useDeviceStore = defineStore('vaultInstanceStore', () => {
  const deviceId = ref<string>()

  const platform = computedAsync(() => tauriPlatform())

  const hostname = computedAsync(() => tauriHostname())

  const deviceName = ref<string>()

  const getDeviceIdAsync = async () => {
    const store = await getStoreAsync()
    return store.get<string>('id')
  }

  const getStoreAsync = async () => {
    const {
      public: { haexVault },
    } = useRuntimeConfig()

    return await load(haexVault.instanceFileName || 'instance.json')
  }

  const setDeviceIdAsync = async (id?: string) => {
    const store = await getStoreAsync()
    const _id = id || crypto.randomUUID()
    await store.set('id', _id)
    deviceId.value = _id
    return _id
  }

  const setDeviceIdIfNotExistsAsync = async () => {
    const _deviceId = await getDeviceIdAsync()
    if (_deviceId) {
      deviceId.value = _deviceId
      return deviceId.value
    }
    return await setDeviceIdAsync()
  }

  const isKnownDeviceAsync = async () => {
    const { readDeviceNameAsync } = useVaultSettingsStore()
    const deviceId = await getDeviceIdAsync()
    return deviceId ? (await readDeviceNameAsync(deviceId)) || false : false
  }

  const readDeviceNameAsync = async (id?: string) => {
    const { readDeviceNameAsync } = useVaultSettingsStore()
    const _id = id ?? deviceId.value
    console.log('readDeviceNameAsync id', _id)
    if (!_id) return

    deviceName.value = (await readDeviceNameAsync(_id))?.value ?? ''
    return deviceName.value
  }

  const updateDeviceNameAsync = async ({
    id,
    name,
  }: {
    id?: string
    name?: string
  }) => {
    const { updateDeviceNameAsync } = useVaultSettingsStore()
    const _id = id ?? deviceId.value
    if (!_id || !name) return

    deviceName.value = name

    return updateDeviceNameAsync({
      deviceId: _id,
      deviceName: name,
    })
  }

  const addDeviceNameAsync = async ({
    id,
    name,
  }: {
    id?: string
    name: string
  }) => {
    const { addDeviceNameAsync } = useVaultSettingsStore()
    const _id = id ?? deviceId.value
    if (!_id || !name) throw new Error('Id oder Name fehlen')

    return addDeviceNameAsync({
      deviceId: _id,
      deviceName: name,
    })
  }
  return {
    addDeviceNameAsync,
    deviceId,
    deviceName,
    hostname,
    isKnownDeviceAsync,
    platform,
    readDeviceNameAsync,
    setDeviceIdAsync,
    setDeviceIdIfNotExistsAsync,
    updateDeviceNameAsync,
  }
})

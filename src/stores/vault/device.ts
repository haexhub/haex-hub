import { load } from '@tauri-apps/plugin-store'
import {
  hostname as tauriHostname,
  platform as tauriPlatform,
} from '@tauri-apps/plugin-os'

const deviceIdKey = 'deviceId'
const defaultDeviceFileName = 'device.json'

export const useDeviceStore = defineStore('vaultDeviceStore', () => {
  const deviceId = ref<string | undefined>('')

  const syncDeviceIdAsync = async () => {
    deviceId.value = await getDeviceIdAsync()
    if (deviceId.value) return deviceId.value

    deviceId.value = await setDeviceIdAsync()
  }

  const platform = computedAsync(() => tauriPlatform())

  const hostname = computedAsync(() => tauriHostname())

  const deviceName = ref<string>()

  const getDeviceIdAsync = async () => {
    const store = await getStoreAsync()
    return await store.get<string>(deviceIdKey)
  }

  const getStoreAsync = async () => {
    const {
      public: { haexVault },
    } = useRuntimeConfig()

    return await load(haexVault.deviceFileName || defaultDeviceFileName)
  }

  const setDeviceIdAsync = async (id?: string) => {
    const store = await getStoreAsync()
    const _id = id || crypto.randomUUID()
    await store.set(deviceIdKey, _id)
    return _id
  }

  const isKnownDeviceAsync = async () => {
    const { readDeviceNameAsync } = useVaultSettingsStore()
    return !!(await readDeviceNameAsync(deviceId.value))
  }

  const readDeviceNameAsync = async (id?: string) => {
    const { readDeviceNameAsync } = useVaultSettingsStore()
    const _id = id || deviceId.value

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
    getDeviceIdAsync,
    hostname,
    isKnownDeviceAsync,
    platform,
    readDeviceNameAsync,
    setDeviceIdAsync,
    syncDeviceIdAsync,
    updateDeviceNameAsync,
  }
})

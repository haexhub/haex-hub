import { load, Store } from '@tauri-apps/plugin-store';

interface ILastVault {
  lastUsed: Date;
  name: string;
  path: string;
}

export const useLastVaultStore = defineStore('lastVaultStore', () => {
  const {
    public: { haexVault },
  } = useRuntimeConfig();

  const lastVaults = ref<ILastVault[]>([]);

  const keyName = 'lastVaults';

  const getStoreAsync = async () => {
    return await load(haexVault.lastVaultFileName);
  };

  const syncLastVaultsAsync = async () => {
    const store = await getStoreAsync();
    lastVaults.value =
      (await store.get<ILastVault[]>(keyName))?.sort(
        (a, b) => +new Date(b.lastUsed) - +new Date(a.lastUsed)
      ) ?? [];

    return lastVaults.value;
  };

  const addVaultAsync = async ({
    name,
    path,
  }: {
    name?: string;
    path: string;
  }) => {
    if (!lastVaults.value) await syncLastVaultsAsync();

    const saveName = name || getFileNameFromPath(path);
    lastVaults.value = lastVaults.value.filter((vault) => vault.path !== path);
    lastVaults.value.push({ lastUsed: new Date(), name: saveName, path });
    await saveLastVaultsAsync();
  };

  const removeVaultAsync = async (vaultPath: string) => {
    console.log('remove', vaultPath, lastVaults.value);
    lastVaults.value = lastVaults.value.filter(
      (vault) => vault.path !== vaultPath
    );
    await saveLastVaultsAsync();
  };

  const saveLastVaultsAsync = async () => {
    const store = await getStoreAsync();
    console.log('save lastVaults', keyName, lastVaults.value);
    await store.set(keyName, lastVaults.value);
    await syncLastVaultsAsync();
  };

  return {
    addVaultAsync,
    syncLastVaultsAsync,
    lastVaults,
    removeVaultAsync,
    saveLastVaultsAsync,
  };
});

const getFileNameFromPath = (path: string) => {
  const lastBackslashIndex = path.lastIndexOf('\\');
  const lastSlashIndex = path.lastIndexOf('/');

  const lastIndex = Math.max(lastBackslashIndex, lastSlashIndex);

  const fileName = path.substring(lastIndex + 1);

  return fileName;
};

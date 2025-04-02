//import Database from '@tauri-apps/plugin-sql';
import { drizzle, SqliteRemoteDatabase } from 'drizzle-orm/sqlite-proxy';
//import Database from "tauri-plugin-sql-api";
import * as schema from '@/../src-tauri/database/schemas/vault';

import { invoke } from '@tauri-apps/api/core';
import { count } from 'drizzle-orm';
import { platform } from '@tauri-apps/plugin-os';

interface IVault {
  //database: Database;
  path: string;
  password: string;
  name: string;
  drizzle: SqliteRemoteDatabase<typeof schema>;
}
interface IOpenVaults {
  [vaultPath: string]: IVault;
}

export const useVaultStore = defineStore('vaultStore', () => {
  const currentVaultId = computed<string | undefined>({
    get: () =>
      getSingleRouteParam(useRouter().currentRoute.value.params.vaultId),
    set: (newVaultId) => {
      useRouter().currentRoute.value.params.vaultId = newVaultId ?? '';
    },
  });

  const read_only = computed<boolean>({
    get: () => {
      console.log(
        'query showSidebar',
        useRouter().currentRoute.value.query.readonly
      );
      return JSON.parse(
        getSingleRouteParam(useRouter().currentRoute.value.query.readonly) ||
          'false'
      );
    },
    set: (readonly) => {
      const router = useRouter();
      router.replace({
        query: {
          ...router.currentRoute.value.query,
          readonly: JSON.stringify(readonly ? true : false),
        },
      });
    },
  });

  const openVaults = ref<IOpenVaults | undefined>();

  const currentVault = ref<IVault | undefined>();

  /*  computed(() => {
    console.log('compute currentVault', currentVaultId.value, openVaults.value);
    return openVaults.value?.[currentVaultId.value ?? ''];
  }); */

  watch(
    currentVaultId,
    async () => {
      /* if (!openVaults.value?.[currentVaultId.value ?? '']) {
        console.log(
          'no vaultId',
          currentVault.value,
          openVaults.value?.[currentVaultId.value ?? '']
        );
        return await navigateTo(useLocalePath()({ name: 'vaultOpen' }));
      } else */
      currentVault.value = openVaults.value?.[currentVaultId.value ?? ''];
    },
    { immediate: true }
  );

  const openAsync = async ({
    path = '',
    password,
  }: {
    path: string;
    password: string;
  }) => {
    //const sqlitePath = path?.startsWith('sqlite:') ? path : `sqlite:${path}`;

    console.log('try to open db', path, password);

    const result = await invoke<string>('open_encrypted_database', {
      path,
      key: password,
    });

    console.log('open vault from store', result);
    if (!(await testDatabaseReadAsync())) throw new Error('Passwort falsch');
    //const db = await Database.load(sqlitePath);

    const vaultId = await getVaultIdAsync(path);
    const seperator = platform() === 'windows' ? '\\' : '/';
    const fileName = path.split(seperator).pop();
    console.log('opened db fileName', fileName, vaultId);

    openVaults.value = {
      ...openVaults.value,
      [vaultId]: {
        //database: db,
        path,
        password,
        name: fileName ?? path,
        drizzle: drizzle<typeof schema>(
          async (sql, params, method) => {
            let rows: any = [];
            let results = [];

            // If the query is a SELECT, use the select method
            if (isSelectQuery(sql)) {
              rows = await invoke('db_select', { sql, params }).catch((e) => {
                console.error('SQL Error:', e);
                return [];
              });
            } else {
              // Otherwise, use the execute method
              rows = await invoke('db_execute', { sql, params }).catch((e) => {
                console.error('SQL Error:', e);
                return [];
              });
              return { rows: [] };
            }

            rows = rows.map((row: any) => {
              return Object.values(row);
            });

            // If the method is "all", return all rows
            results = method === 'all' ? rows : rows[0];

            return { rows: results };
          },
          // Pass the schema to the drizzle instance
          { schema: schema, logger: true }
        ),
      },
    };

    const { addVaultAsync } = useLastVaultStore();
    await addVaultAsync({ path });

    return vaultId;
  };

  const testDatabaseReadAsync = async () => {
    try {
      currentVault.value?.drizzle
        .select({ count: count() })
        .from(schema.haexExtensions);
      return true;
    } catch (error) {
      return false;
    }
  };

  const refreshDatabaseAsync = async () => {
    console.log('refreshDatabaseAsync');
    /*     if (!currentVault.value?.database.close) {
      return navigateTo(useLocaleRoute()({ name: 'vaultOpen' }));
    } */
  };

  const createAsync = async ({
    path,
    password,
  }: {
    path: string;
    password: string;
  }) => {
    /* const existDb = await exists('default.db', {
        baseDir: BaseDirectory.Resource,
      }); */

    /* const existDb = await resolveResource('resources/default.db');
    if (!existDb) throw new Error('Keine Datenbank da');
    await copyFile(existDb, path); */
    const result = await invoke('create_encrypted_database', {
      path,
      key: password,
    });
    console.log('create_encrypted_database', result);
    return 'aaaaa'; //await openAsync({ path, password });
  };

  const closeAsync = async () => {
    if (!currentVaultId.value) return;

    /* if (
      typeof openVaults.value?.[currentVaultId.value]?.database?.close ===
      'function'
    ) {
      console.log('close db', openVaults.value?.[currentVaultId.value]);
      return openVaults.value?.[currentVaultId.value]?.database?.close();
    } */
    delete openVaults.value?.[currentVaultId.value];
  };

  return {
    closeAsync,
    createAsync,
    currentVault,
    currentVaultId,
    openAsync,
    openVaults,
    refreshDatabaseAsync,
    read_only,
  };
});

const getVaultIdAsync = async (path: string) => {
  const encoder = new TextEncoder();
  const data = encoder.encode(path);

  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer)); // convert buffer to byte array
  const hashHex = hashArray
    .map((b) => b.toString(16).padStart(2, '0'))
    .join(''); // convert bytes to hex string
  console.log('vaultId', hashHex);
  return hashHex;
};

function isSelectQuery(sql: string): boolean {
  const selectRegex = /^\s*SELECT\b/i;
  return selectRegex.test(sql);
}

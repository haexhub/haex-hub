import { eq, isNull } from 'drizzle-orm'
import {
  haexPasswordsGroupItems,
  haexPasswordsGroups,
  haexPasswordsItems,
  type InsertHaexPasswordsItems,
  type InsertHaexPasswordsItemsKeyValues,
} from '~~/src-tauri/database/schemas/vault'

export const usePasswordItemStore = defineStore('passwordItemStore', () => {
  const currentItemId = computed({
    get: () =>
      getSingleRouteParam(useRouter().currentRoute.value.params.itemId),
    set: (entryId) => {
      console.log('set entryId', entryId)
      useRouter().currentRoute.value.params.entryId = entryId ?? ''
    },
  })

  return {
    currentItemId,
    addAsync,
    readByGroupIdAsync,
    readAsync,
    readKeyValuesAsync,
  }
})

const addAsync = async (
  item: InsertHaexPasswordsItems,
  keyValues: InsertHaexPasswordsItemsKeyValues,
) => {
  const { currentVault } = useVaultStore()
  /* const { currentGroupId } = useVaultGroupStore();

  entry.id = crypto.randomUUID();
  entry.createdAt = null;
  entry.updateAt = null;
  console.log('store create entry', entry, currentGroupId);
  await currentVault?.drizzle.transaction(async (tx) => {
    await tx.insert(vaultEntry).values(entry);
    await tx
      .insert(vaultGroupEntry)
      .values({ entryId: entry.id, groupId: currentGroupId });
  });

  return entry.id; */
}

const readByGroupIdAsync = async (groupId?: string | null) => {
  try {
    const { currentVault } = useVaultStore()

    console.log('get entries by groupId', groupId || null)

    if (groupId) {
      const entries = await currentVault.drizzle
        .select()
        .from(haexPasswordsGroupItems)
        .innerJoin(
          haexPasswordsItems,
          eq(haexPasswordsItems.id, haexPasswordsGroupItems.itemId),
        )
        .where(eq(haexPasswordsGroupItems.groupId, groupId))

      console.log('found entries by groupId', entries)
      return entries.map((entry) => entry.haex_passwords_items)
    } else {
      const entries = await currentVault.drizzle
        .select()
        .from(haexPasswordsGroupItems)
        .innerJoin(
          haexPasswordsItems,
          eq(haexPasswordsItems.id, haexPasswordsGroupItems.itemId),
        )
        .where(isNull(haexPasswordsGroupItems.groupId))

      console.log('found entries', entries)
      return entries.map((entry) => entry.haex_passwords_items)
    }
  } catch (error) {
    console.error(error)
    return []
  }
}

const readAsync = async (itemId: string | null) => {
  if (!itemId) return null

  try {
    const { currentVault } = useVaultStore()

    const details =
      await currentVault.drizzle.query.haexPasswordsItems.findFirst({
        where: eq(haexPasswordsItems.id, itemId),
      })

    if (!details) return {}

    const history = (await usePasswordHistoryStore().getAsync(itemId)) ?? []
    const keyValues = (await readKeyValuesAsync(itemId)) ?? []

    console.log('found item by id', { details, history, keyValues })
    return { details, history, keyValues }
  } catch (error) {
    console.error(error)
    throw error
  }
}

const readKeyValuesAsync = async (itemId: string | null) => {
  if (!itemId) return null
  const { currentVault } = useVaultStore()

  const keyValues =
    await currentVault.drizzle.query.haexPasswordsItemsKeyValues.findMany({
      where: eq(haexPasswordsItems.id, itemId),
    })
  return keyValues
}

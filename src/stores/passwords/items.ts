import { eq, isNull } from 'drizzle-orm'
import {
  haexPasswordsGroupItems,
  haexPasswordsItemDetails,
  haexPasswordsItemHistory,
  haexPasswordsItemKeyValues,
  type InsertHaexPasswordsItemDetails,
  type InserthaexPasswordsItemKeyValues,
  type SelectHaexPasswordsGroups,
  type SelectHaexPasswordsItemDetails,
  type SelectHaexPasswordsItemKeyValues,
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

  const currentItem = computedAsync(() => readAsync(currentItemId.value))

  return {
    currentItemId,
    currentItem,
    addAsync,
    addKeyValueAsync,
    addKeyValuesAsync,
    deleteAsync,
    deleteKeyValueAsync,
    readByGroupIdAsync,
    readAsync,
    readKeyValuesAsync,
    updateAsync,
  }
})

const addAsync = async (
  details: SelectHaexPasswordsItemDetails,
  keyValues: SelectHaexPasswordsItemKeyValues[],
  group?: SelectHaexPasswordsGroups | null,
) => {
  const { currentVault } = useVaultStore()
  console.log('addItem', details, group)

  const newDetails: InsertHaexPasswordsItemDetails = {
    id: crypto.randomUUID(),
    icon: details.icon || group?.icon || null,
    note: details.note,
    password: details.password,
    tags: details.tags,
    title: details.title,
    url: details.url,
    username: details.username,
  }

  const newKeyValues: InserthaexPasswordsItemKeyValues[] = keyValues.map(
    (keyValue) => ({
      id: crypto.randomUUID(),
      itemId: newDetails.id,
      key: keyValue.key,
      value: keyValue.value,
    }),
  )

  try {
    await currentVault?.drizzle.transaction(async (tx) => {
      await tx.insert(haexPasswordsItemDetails).values(newDetails)

      await tx
        .insert(haexPasswordsGroupItems)
        .values({ itemId: newDetails.id, groupId: group?.id ?? null })

      if (newKeyValues.length)
        await tx.insert(haexPasswordsItemKeyValues).values(newKeyValues)
    })
  } catch (error) {
    console.error('ERROR addItem', error)
  }

  return newDetails.id
}

const addKeyValueAsync = async (
  item?: InserthaexPasswordsItemKeyValues | null,
  itemId?: string,
) => {
  const newKeyValue: InserthaexPasswordsItemKeyValues = {
    id: crypto.randomUUID(),
    itemId: item?.itemId || itemId,
    key: item?.key,
    value: item?.value,
  }
  try {
    const { currentVault } = useVaultStore()
    return await currentVault?.drizzle
      .insert(haexPasswordsItemKeyValues)
      .values(newKeyValue)
  } catch (error) {
    console.error('ERROR addItem', error)
  }
}

const addKeyValuesAsync = async (
  items: InserthaexPasswordsItemKeyValues[],
  itemId?: string,
) => {
  const { currentVault } = useVaultStore()
  console.log('addKeyValues', items, itemId)

  const newKeyValues: InserthaexPasswordsItemKeyValues[] = items?.map(
    (item) => ({
      id: crypto.randomUUID(),
      itemId: item.itemId || itemId,
      key: item.key,
      value: item.value,
    }),
  )

  try {
    return await currentVault?.drizzle
      .insert(haexPasswordsItemKeyValues)
      .values(newKeyValues)
  } catch (error) {
    console.error('ERROR addItem', error)
  }
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
          haexPasswordsItemDetails,
          eq(haexPasswordsItemDetails.id, haexPasswordsGroupItems.itemId),
        )
        .where(eq(haexPasswordsGroupItems.groupId, groupId))

      console.log('found entries by groupId', entries)
      return entries.map((entry) => entry.haex_passwords_item_details)
    } else {
      const entries = await currentVault.drizzle
        .select()
        .from(haexPasswordsGroupItems)
        .innerJoin(
          haexPasswordsItemDetails,
          eq(haexPasswordsItemDetails.id, haexPasswordsGroupItems.itemId),
        )
        .where(isNull(haexPasswordsGroupItems.groupId))

      console.log('found entries', entries)
      return entries.map((entry) => entry.haex_passwords_item_details)
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
      await currentVault.drizzle.query.haexPasswordsItemDetails.findFirst({
        where: eq(haexPasswordsItemDetails.id, itemId),
      })

    console.log('readAsync details', details)

    if (!details) return null

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
    await currentVault.drizzle.query.haexPasswordsItemKeyValues.findMany({
      where: eq(haexPasswordsGroupItems.itemId, itemId),
    })
  return keyValues
}

const updateAsync = async ({
  details,
  keyValues,
  keyValuesAdd,
  keyValuesDelete,
  groupId,
}: {
  details: SelectHaexPasswordsItemDetails
  keyValues: SelectHaexPasswordsItemKeyValues[]
  keyValuesAdd: SelectHaexPasswordsItemKeyValues[]
  keyValuesDelete: SelectHaexPasswordsItemKeyValues[]
  groupId: string | null
}) => {
  const { currentVault } = useVaultStore()

  if (!details.id) return

  const newDetails: InsertHaexPasswordsItemDetails = {
    id: details.id,
    icon: details.icon,
    note: details.note,
    password: details.password,
    tags: details.tags,
    title: details.title,
    url: details.url,
    username: details.username,
  }

  const newKeyValues: InserthaexPasswordsItemKeyValues[] = keyValues
    .map((keyValue) => ({
      id: keyValue.id,
      itemId: newDetails.id,
      key: keyValue.key,
      value: keyValue.value,
    }))
    .filter((keyValue) => keyValue.id)

  const newKeyValuesAdd: InserthaexPasswordsItemKeyValues[] = keyValuesAdd.map(
    (keyValue) => ({
      id: keyValue.id || crypto.randomUUID(),
      itemId: newDetails.id,
      key: keyValue.key,
      value: keyValue.value,
    }),
  )

  console.log('update item', newDetails, newKeyValues, newKeyValuesAdd, groupId)

  return await currentVault?.drizzle.transaction(async (tx) => {
    await tx
      .update(haexPasswordsItemDetails)
      .set(newDetails)
      .where(eq(haexPasswordsItemDetails.id, newDetails.id))

    await tx
      .update(haexPasswordsGroupItems)
      .set({ itemId: newDetails.id, groupId })
      .where(eq(haexPasswordsGroupItems.itemId, newDetails.id))

    const promises = newKeyValues.map((keyValue) =>
      tx
        .update(haexPasswordsItemKeyValues)
        .set(keyValue)
        .where(eq(haexPasswordsItemKeyValues.id, keyValue.id)),
    )

    await Promise.all(promises)

    if (newKeyValuesAdd.length)
      await tx.insert(haexPasswordsItemKeyValues).values(newKeyValuesAdd)

    const promisesDelete = keyValuesDelete.map((keyValue) =>
      tx
        .delete(haexPasswordsItemKeyValues)
        .where(eq(haexPasswordsItemKeyValues.id, keyValue.id)),
    )
    await Promise.all(promisesDelete)

    return newDetails.id
  })
}

const deleteAsync = async (itemId: string, final: boolean = false) => {
  const { currentVault } = useVaultStore()
  const { createTrashIfNotExistsAsync, trashId } = usePasswordGroupStore()

  console.log('deleteAsync', itemId, final)
  if (final)
    await currentVault?.drizzle.transaction(async (tx) => {
      await tx
        .delete(haexPasswordsItemKeyValues)
        .where(eq(haexPasswordsItemKeyValues.itemId, itemId))
      await tx
        .delete(haexPasswordsItemHistory)
        .where(eq(haexPasswordsItemHistory.itemId, itemId))
      await tx
        .delete(haexPasswordsGroupItems)
        .where(eq(haexPasswordsGroupItems.itemId, itemId))
      await tx
        .delete(haexPasswordsItemDetails)
        .where(eq(haexPasswordsItemDetails.id, itemId))
    })
  else {
    if (await createTrashIfNotExistsAsync())
      await currentVault.drizzle
        .update(haexPasswordsGroupItems)
        .set({ groupId: trashId })
        .where(eq(haexPasswordsGroupItems.itemId, itemId))
  }
}

const deleteKeyValueAsync = async (id: string) => {
  console.log('deleteKeyValueAsync', id)
  const { currentVault } = useVaultStore()
  return await currentVault.drizzle
    .delete(haexPasswordsItemKeyValues)
    .where(eq(haexPasswordsItemKeyValues.id, id))
}

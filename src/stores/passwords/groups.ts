import { and, eq, isNull, sql, type SQLWrapper } from 'drizzle-orm'
import {
  haexPasswordsGroupItems,
  haexPasswordsGroups,
  haexPasswordsItems,
  type InsertHaexPasswordsGroups,
  type InsertHaexPasswordsItems,
  type SelectHaexPasswordsGroups,
  type SelectHaexPasswordsItems,
} from '~~/src-tauri/database/schemas/vault'

export const usePasswordGroupStore = defineStore('passwordGroupStore', () => {
  const groups = ref<SelectHaexPasswordsGroups[]>([])

  const currentGroupId = computed<string | null>({
    get: () =>
      getSingleRouteParam(useRouter().currentRoute.value.params.groupId) ||
      null,
    set: (newGroupId) => {
      console.log('set groupId', newGroupId)
      useRouter().currentRoute.value.params.groupId = newGroupId ?? ''
    },
  })

  const currentGroup = ref()

  const currentGroupItems = reactive<{
    items: SelectHaexPasswordsItems[]
    groups: SelectHaexPasswordsGroups[]
  }>({
    items: [],
    groups: [],
  })

  const syncGroupItemsAsync = async (currentGroupId: string | null) => {
    const { addNotificationAsync } = useNotificationStore()
    const { readByGroupIdAsync } = usePasswordItemStore()
    /*  const { currentGroup, groups, currentGroupItems } = storeToRefs(
      usePasswordGroupStore(),
    ) */
    groups.value = await readGroupsAsync()
    currentGroup.value = groups.value?.find(
      (group) => group.id === currentGroupId,
    )
    try {
      currentGroupItems.groups =
        (await getByParentIdAsync(currentGroupId)) ?? []
      currentGroupItems.items = (await readByGroupIdAsync(currentGroupId)) ?? []
      console.log('search current group', groups.value, currentGroup.value)
    } catch (error) {
      console.error(error)
      currentGroupItems.groups = []
      currentGroupItems.items = []
      await addNotificationAsync({
        type: 'log',
        text: JSON.stringify(error),
      })
    }
  }
  watch(
    currentGroupId,
    async () => {
      syncGroupItemsAsync(currentGroupId.value)
    },
    { immediate: true },
  )

  return {
    addGroupAsync,
    currentGroup,
    currentGroupId,
    currentGroupItems,
    groups,
    navigateToGroupAsync,
    navigateToGroupItemsAsync,
    readGroupAsync,
    readGroupItemsAsync,
    readGroupsAsync,
    updateAsync,
  }
})

const addGroupAsync = async (group: Partial<InsertHaexPasswordsGroups>) => {
  const { currentVault } = useVaultStore()

  const newGroup: InsertHaexPasswordsGroups = {
    id: crypto.randomUUID(),
    parentId: group.parentId,
    color: group.color,
    icon: group.icon,
    name: group.name,
    order: group.order,
  }
  await currentVault.drizzle.insert(haexPasswordsGroups).values(newGroup)
  return newGroup
}

const readGroupAsync = async (groupId: string) => {
  const { currentVault } = useVaultStore()

  return (
    await currentVault.drizzle
      .select()
      .from(haexPasswordsGroups)
      .where(eq(haexPasswordsGroups.id, groupId))
  ).at(0)
}

const readGroupsAsync = async (filter?: { parentId?: string | null }) => {
  const { currentVault } = storeToRefs(useVaultStore())
  if (filter?.parentId) {
    return await currentVault.value.drizzle
      .select()
      .from(haexPasswordsGroups)
      .where(eq(haexPasswordsGroups.id, filter.parentId))
  } else {
    return await currentVault.value.drizzle
      .select()
      .from(haexPasswordsGroups)
      .where(isNull(haexPasswordsGroups.parentId))
      .orderBy(sql`${haexPasswordsGroups.order} nulls last`)
  }
}

const readGroupItemsAsync = async (id?: string | null) => {
  const { currentVault } = useVaultStore()

  currentVault.drizzle.select().from(haexPasswordsGroupItems)
}

const getByParentIdAsync = async (parentId?: string | null) => {
  try {
    const { currentVault } = useVaultStore()

    if (parentId) {
      const groups = await currentVault.drizzle
        .select()
        .from(haexPasswordsGroups)
        .where(eq(haexPasswordsGroups.parentId, parentId))
        .orderBy(sql`${haexPasswordsGroups.order} nulls last`)

      console.log('found groups', groups)
      return groups
    } else {
      const groups = await currentVault.drizzle
        .select()
        .from(haexPasswordsGroups)
        .where(isNull(haexPasswordsGroups.parentId))
        .orderBy(sql`${haexPasswordsGroups.order} nulls last`)

      console.log('found groups', groups)
      return groups
    }
  } catch (error) {
    console.error(error)
  }
}

const navigateToGroupAsync = (groupId?: string | null) =>
  navigateTo(
    useLocaleRoute()({
      name: 'passwordGroupEdit',
      params: {
        vaultId: useRouter().currentRoute.value.params.vaultId,
        groupId,
      },
      query: {
        ...useRouter().currentRoute.value.query,
      },
    }),
  )

const updateAsync = async () => {}

const navigateToGroupItemsAsync = (groupId: string) => {
  navigateTo(
    useLocaleRoute()({
      name: 'passwordGroupItems',
      params: {
        vaultId: useRouter().currentRoute.value.params.vaultId,
        groupId,
      },
      query: {
        ...useRouter().currentRoute.value.query,
      },
    }),
  )
}

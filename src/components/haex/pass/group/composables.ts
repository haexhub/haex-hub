import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

export const usePasswordGroup = () => {
  const areItemsEqual = (
    groupA: unknown | unknown[] | null,
    groupB: unknown | unknown[] | null,
  ) => {
    if (groupA === null && groupB === null) return true

    if (Array.isArray(groupA) && Array.isArray(groupB)) {
      console.log('compare object arrays', groupA, groupB)
      if (groupA.length === groupB.length) return true

      return groupA.some((group, index) => {
        return areObjectsEqual(group, groupA[index])
      })
    }
    return areObjectsEqual(groupA, groupB)
  }

  return {
    areItemsEqual,
  }
}

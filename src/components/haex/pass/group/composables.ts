export const usePasswordGroup = () => {
  const areItemsEqual = (
    groupA: unknown | unknown[] | null,
    groupB: unknown | unknown[] | null,
  ) => {
    console.log('compare values', groupA, groupB)
    if (groupA === groupB) return true

    if (Array.isArray(groupA) && Array.isArray(groupB)) {
      console.log('compare object arrays', groupA, groupB)
      if (groupA.length === groupB.length) return true

      return groupA.some((group, index) => {
        return areObjectsEqual(group, groupA[index])
      })
    }
    return areObjectsEqual(groupA, groupB)
  }

  const deepEqual = (obj1: unknown, obj2: unknown) => {
    console.log('compare values', obj1, obj2)
    if (obj1 === obj2) return true

    // Null/undefined Check
    if (obj1 == null || obj2 == null) return obj1 === obj2

    // Typ-Check
    if (typeof obj1 !== typeof obj2) return false

    // Primitive Typen
    if (typeof obj1 !== 'object') return obj1 === obj2

    // Arrays
    if (Array.isArray(obj1) !== Array.isArray(obj2)) return false

    if (Array.isArray(obj1)) {
      if (obj1.length !== obj2.length) return false
      for (let i = 0; i < obj1.length; i++) {
        if (!deepEqual(obj1[i], obj2[i])) return false
      }
      return true
    }

    // Date Objekte
    if (obj1 instanceof Date && obj2 instanceof Date) {
      return obj1.getTime() === obj2.getTime()
    }

    // RegExp Objekte
    if (obj1 instanceof RegExp && obj2 instanceof RegExp) {
      return obj1.toString() === obj2.toString()
    }

    // Objekte
    const keys1 = Object.keys(obj1)
    const keys2 = Object.keys(obj2)

    if (keys1.length !== keys2.length) return false

    for (const key of keys1) {
      if (!keys2.includes(key)) return false
      if (!deepEqual(obj1[key], obj2[key])) return false
    }

    return true
  }
  return {
    areItemsEqual,
    deepEqual,
  }
}

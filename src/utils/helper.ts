import type { LocationQueryValue, RouteLocationRawI18n } from 'vue-router'

export const bytesToBase64DataUrlAsync = async (
  bytes: Uint8Array,
  type = 'application/octet-stream',
) => {
  return await new Promise((resolve, reject) => {
    const reader = Object.assign(new FileReader(), {
      onload: () => resolve(reader.result),
      onerror: () => reject(reader.error),
    })
    reader.readAsDataURL(new File([new Blob([bytes])], '', { type }))
  })
}

export const blobToImageAsync = (blob: Blob): Promise<HTMLImageElement> => {
  return new Promise((resolve) => {
    console.log('transform blob', blob)
    const url = URL.createObjectURL(blob)
    const img = new Image()
    img.onload = () => {
      URL.revokeObjectURL(url)
      resolve(img)
    }
    img.src = url
  })
}

export const deepToRaw = <T extends Record<string, any>>(sourceObj: T): T => {
  const objectIterator = (input: any): any => {
    if (Array.isArray(input)) {
      return input.map((item) => objectIterator(item))
    }
    if (isRef(input) || isReactive(input) || isProxy(input)) {
      return objectIterator(toRaw(input))
    }
    if (input && typeof input === 'object') {
      return Object.keys(input).reduce((acc, key) => {
        acc[key as keyof typeof acc] = objectIterator(input[key])
        return acc
      }, {} as T)
    }
    return input
  }

  return objectIterator(sourceObj)
}

export const readableFileSize = (sizeInByte: number | string = 0) => {
  if (!sizeInByte) {
    return '0 KB'
  }
  const size =
    typeof sizeInByte === 'string' ? parseInt(sizeInByte) : sizeInByte
  const sizeInKb = size / 1024
  const sizeInMb = sizeInKb / 1024
  const sizeInGb = sizeInMb / 1024
  const sizeInTb = sizeInGb / 1024

  if (sizeInTb > 1) return `${sizeInTb.toFixed(2)} TB`
  if (sizeInGb > 1) return `${sizeInGb.toFixed(2)} GB`
  if (sizeInMb > 1) return `${sizeInMb.toFixed(2)} MB`

  return `${sizeInKb.toFixed(2)} KB`
}

export const getSingleRouteParam = (
  param: string | string[] | LocationQueryValue | LocationQueryValue[],
): string => {
  const _param = Array.isArray(param) ? param.at(0) ?? '' : param ?? ''
  //console.log('getSingleRouteParam found:', _param, param)
  return decodeURIComponent(_param)
}

export const isRouteActive = (
  to: RouteLocationRawI18n,
  exact: boolean = false,
) =>
  computed(() => {
    const found = useRouter()
      .getRoutes()
      .find((route) => route.name === useLocaleRoute()(to)?.name)
    //console.log('found route', found, useRouter().currentRoute.value, to);
    return exact
      ? found?.name === useRouter().currentRoute.value.name
      : found?.name === useRouter().currentRoute.value.name ||
          found?.children.some(
            (child) => child.name === useRouter().currentRoute.value.name,
          )
  })

export const isKey = <T extends object>(x: T, k: PropertyKey): k is keyof T => {
  return k in x
}

export const filterAsync = async <T>(
  arr: T[],
  predicate: (value: T, index: number, array: T[]) => Promise<boolean>,
) => {
  // 1. Mappe jedes Element auf ein Promise, das zu true/false auflöst
  const results = await Promise.all(arr.map(predicate))

  // 2. Filtere das ursprüngliche Array basierend auf den Ergebnissen
  return arr.filter((_value, index) => results[index])
}

export const stringToHex = (str: string) =>
  str
    .split('')
    .map((char) => char.charCodeAt(0).toString(16).padStart(2, '0'))
    .join('') // Join array into a single string

export const hexToString = (hex: string) => {
  if (!hex) return ''
  const parsedValue = hex
    .match(/.{1,2}/g) // Split hex into pairs
    ?.map((byte) => String.fromCharCode(parseInt(byte, 16))) // Convert hex to char
    .join('') // Join array into a single string

  return parsedValue ? parsedValue : ''
}

export const getContrastingTextColor = (
  hexColor: string,
): 'black' | 'white' => {
  if (!hexColor) {
    return 'black' // Fallback
  }

  // Entferne das '#' vom Anfang
  let color = hexColor.startsWith('#') ? hexColor.slice(1) : hexColor

  // Handle Kurzform-Hex-Werte (z.B. "F0C" -> "FF00CC")
  if (color.length === 3) {
    color = color
      .split('')
      .map((char) => char + char)
      .join('')
  }

  if (color.length !== 6) {
    return 'black' // Fallback für ungültige Farben
  }

  // Konvertiere Hex zu RGB
  const r = parseInt(color.substring(0, 2), 16)
  const g = parseInt(color.substring(2, 4), 16)
  const b = parseInt(color.substring(4, 6), 16)

  // Berechne die wahrgenommene Luminanz nach der WCAG-Formel.
  // Werte von 0 (schwarz) bis 255 (weiß).
  const luminance = 0.299 * r + 0.587 * g + 0.114 * b

  // Wähle die Textfarbe basierend auf einem Schwellenwert.
  // Ein Wert > 186 wird oft als "hell" genug für schwarzen Text angesehen.
  return luminance > 186 ? 'black' : 'white'
}

import { platform, type Platform } from '@tauri-apps/plugin-os'

let cachedPlatform: Platform | null = null

/**
 * Get the current platform (cached after first call)
 */
export const getPlatform = (): Platform => {
  if (!cachedPlatform) {
    cachedPlatform = platform()
  }
  return cachedPlatform
}

/**
 * Check if running on a desktop platform (Windows, Linux, macOS)
 */
export const isDesktop = (): boolean => {
  const p = getPlatform()
  return p === 'windows' || p === 'linux' || p === 'macos'
}

/**
 * Check if running on a mobile platform (Android, iOS)
 */
export const isMobile = (): boolean => {
  const p = getPlatform()
  return p === 'android' || p === 'ios'
}

/**
 * Check if running on Android
 */
export const isAndroid = (): boolean => {
  return getPlatform() === 'android'
}

/**
 * Check if running on iOS
 */
export const isIOS = (): boolean => {
  return getPlatform() === 'ios'
}

/**
 * Check if running on Windows
 */
export const isWindows = (): boolean => {
  return getPlatform() === 'windows'
}

/**
 * Check if running on Linux
 */
export const isLinux = (): boolean => {
  return getPlatform() === 'linux'
}

/**
 * Check if running on macOS
 */
export const isMacOS = (): boolean => {
  return getPlatform() === 'macos'
}

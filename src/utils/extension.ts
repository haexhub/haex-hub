/**
 * Utility functions for working with HaexHub extensions
 */

import { platform } from '@tauri-apps/plugin-os'
import {
  EXTENSION_PROTOCOL_PREFIX,
  EXTENSION_PROTOCOL_NAME,
} from '~/config/constants'

/**
 * Generates the extension URL for loading an extension in an iframe
 *
 * @param publicKey - The extension's public key (64 hex chars)
 * @param name - The extension name
 * @param version - The extension version
 * @param assetPath - Optional asset path (defaults to 'index.html')
 * @param devServerUrl - Optional dev server URL for development extensions
 * @returns The complete extension URL
 */
export async function getExtensionUrl(
  publicKey: string,
  name: string,
  version: string,
  assetPath: string = 'index.html',
  devServerUrl?: string,
): Promise<string> {
  if (!publicKey || !name || !version) {
    console.error('Missing required extension fields')
    return ''
  }

  // If dev server URL is provided, load directly from dev server
  if (devServerUrl) {
    const cleanUrl = devServerUrl.replace(/\/$/, '') // Remove trailing slash
    const cleanPath = assetPath.replace(/^\//, '') // Remove leading slash
    return cleanPath ? `${cleanUrl}/${cleanPath}` : cleanUrl
  }

  // Production extension: Use custom protocol
  // Encode extension info as base64 for unique origin per extension
  const extensionInfo = {
    name,
    publicKey,
    version,
  }
  const encodedInfo = btoa(JSON.stringify(extensionInfo))

  const os = await platform()

  if (os === 'android') {
    // Android: Tauri uses http://{scheme}.localhost format
    return `http://${EXTENSION_PROTOCOL_NAME}.localhost/${encodedInfo}/${assetPath}`
  } else {
    // All other platforms: Use custom protocol
    return `${EXTENSION_PROTOCOL_PREFIX}${encodedInfo}/${assetPath}`
  }
}

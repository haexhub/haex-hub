import type { IHaexHubExtension } from '~/types/haexhub'
import type { ExtensionRequest } from './types'

export async function handlePermissionsMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  if (!extension || !request) {
    throw new Error('Extension not found')
  }

  // TODO: Implementiere Permission Request UI
  throw new Error('Permission methods not yet implemented')
}

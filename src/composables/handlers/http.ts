import type { IHaexHubExtension } from '~/types/haexhub'
import type { ExtensionRequest } from './types'

export async function handleHttpMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  if (!extension || !request) {
    throw new Error('Extension not found')
  }

  // TODO: Implementiere HTTP Commands im Backend
  throw new Error('HTTP methods not yet implemented')
}

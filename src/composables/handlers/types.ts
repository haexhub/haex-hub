// Shared types for extension message handlers
import type { IHaexHubExtension } from '~/types/haexhub'

export interface ExtensionRequest {
  id: string
  method: string
  params: Record<string, unknown>
  timestamp: number
}

export interface ExtensionInstance {
  extension: IHaexHubExtension
  windowId: string
}

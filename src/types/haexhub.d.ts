export interface IHaexHubExtensionManifest {
  name: string
  id: string
  entry: string
  author: string
  url: string
  version: string
  icon: string
  permissions: {
    database?: {
      read?: string[]
      write?: string[]
      create?: string[]
    }
    http?: string[]
    filesystem?: {
      read?: string[]
      write?: string[]
    }
  }
}

/**
 * Installed extension from database/backend
 */
export interface IHaexHubExtension {
  id: string
  name: string
  version: string
  author: string | null
  icon: string | null
  enabled: boolean
  description: string | null
  homepage: string | null
}

/**
 * Marketplace extension with additional metadata
 * Extends IHaexHubExtension with marketplace-specific fields
 */
export interface IMarketplaceExtension extends Omit<IHaexHubExtension, 'enabled'> {
  downloads: number
  rating: number
  verified: boolean
  tags: string[]
  category: string
  downloadUrl: string
  isInstalled: boolean
}

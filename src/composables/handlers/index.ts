// Export all handler functions
export { handleDatabaseMethodAsync } from './database'
export { handleFilesystemMethodAsync } from './filesystem'
export { handleHttpMethodAsync } from './http'
export { handlePermissionsMethodAsync } from './permissions'
export { handleContextMethodAsync, setContextGetters } from './context'
export { handleStorageMethodAsync } from './storage'

// Export shared types
export type { ExtensionRequest, ExtensionInstance } from './types'

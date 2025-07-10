import { invoke } from '@tauri-apps/api/core'

export const useAndroidStorage = () => {
  const requestStoragePermission = async (): Promise<string> => {
    try {
      return await invoke(
        'plugin:android-fs|openManageAllFilesAccessPermissionSettings',
      )
    } catch (error) {
      console.error('Failed to request storage permission:', error)
      throw error
    }
  }

  const hasStoragePermission = async (): Promise<boolean> => {
    try {
      return await invoke('has_storage_permission')
    } catch (error) {
      console.error('Failed to check storage permission:', error)
      return false
    }
  }

  const getExternalStoragePaths = async (): Promise<string[]> => {
    try {
      return await invoke('get_external_storage_paths')
    } catch (error) {
      console.error('Failed to get storage paths:', error)
      return []
    }
  }

  return {
    requestStoragePermission,
    hasStoragePermission,
    getExternalStoragePaths,
  }
}

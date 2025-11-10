import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { openPath } from '@tauri-apps/plugin-opener'
import { tempDir, join } from '@tauri-apps/api/path'
import type { IHaexHubExtension } from '~/types/haexhub'
import type { ExtensionRequest } from './types'

export async function handleFilesystemMethodAsync(
  request: ExtensionRequest,
  extension: IHaexHubExtension,
) {
  if (!request || !extension) return

  switch (request.method) {
    case 'haextension.fs.saveFile': {
      const params = request.params as {
        data: number[]
        defaultPath?: string
        title?: string
        filters?: Array<{ name: string; extensions: string[] }>
      }

      // Convert number array back to Uint8Array
      const data = new Uint8Array(params.data)

      // Open save dialog
      const filePath = await save({
        defaultPath: params.defaultPath,
        title: params.title || 'Save File',
        filters: params.filters,
      })

      // User cancelled
      if (!filePath) {
        return null
      }

      // Write file
      await writeFile(filePath, data)

      return {
        path: filePath,
        success: true,
      }
    }

    case 'haextension.fs.showImage': {
      // This method is now handled by the frontend using PhotoSwipe
      // We keep it for backwards compatibility but it's a no-op
      return {
        success: true,
        useFrontend: true,
      }
    }

    case 'haextension.fs.openFile': {
      const params = request.params as {
        data: number[]
        fileName: string
        mimeType?: string
      }

      try {
        // Convert number array back to Uint8Array
        const data = new Uint8Array(params.data)

        // Get temp directory and create file path
        const tempDirPath = await tempDir()
        const tempFilePath = await join(tempDirPath, params.fileName)

        // Write file to temp directory
        await writeFile(tempFilePath, data)

        // Open file with system's default viewer
        await openPath(tempFilePath)

        return {
          success: true,
        }
      }
      catch (error) {
        console.error('[Filesystem] Error opening file:', error)
        return {
          success: false,
        }
      }
    }

    default:
      throw new Error(`Unknown filesystem method: ${request.method}`)
  }
}

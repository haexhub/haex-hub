import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
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

    default:
      throw new Error(`Unknown filesystem method: ${request.method}`)
  }
}

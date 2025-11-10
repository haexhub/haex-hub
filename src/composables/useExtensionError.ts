import type { SerializedExtensionError } from '~~/src-tauri/bindings/SerializedExtensionError'

/**
 * Type guard to check if error is a SerializedExtensionError
 */
export function isSerializedExtensionError(error: unknown): error is SerializedExtensionError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'message' in error &&
    'type' in error
  )
}

/**
 * Extract error message from unknown error type
 */
export function getErrorMessage(error: unknown): string {
  if (isSerializedExtensionError(error)) {
    return error.message
  }

  if (error instanceof Error) {
    return error.message
  }

  if (typeof error === 'string') {
    return error
  }

  return String(error)
}

/**
 * Composable for handling extension errors
 */
export function useExtensionError() {
  return {
    isSerializedExtensionError,
    getErrorMessage,
  }
}

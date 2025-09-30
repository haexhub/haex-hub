import type { IHaexHubExtensionLink } from '~/types/haexhub'

interface ExtensionRequest {
  id: string
  method: string
  params: Record<string, unknown>
  timestamp: number
}

interface ExtensionResponse {
  id: string
  result?: unknown
  error?: {
    code: string
    message: string
    details?: unknown
  }
}

export const useExtensionMessageHandler = (
  iframeRef: Ref<HTMLIFrameElement | undefined | null>,
  extension: ComputedRef<IHaexHubExtensionLink | undefined>,
) => {
  const handleMessage = async (event: MessageEvent) => {
    // Security: Only accept messages from our iframe
    if (!iframeRef.value || event.source !== iframeRef.value.contentWindow) {
      return
    }

    const request = event.data as ExtensionRequest

    // Validate request structure
    if (!request.id || !request.method) {
      console.error('Invalid extension request:', request)
      return
    }

    console.log('[HaexHub] Extension request:', request.method, request.params)

    try {
      let result: unknown

      // Route request to appropriate handler
      if (request.method.startsWith('extension.')) {
        result = await handleExtensionMethod(request, extension)
      } else if (request.method.startsWith('db.')) {
        result = await handleDatabaseMethod(request, extension)
      } else if (request.method.startsWith('permissions.')) {
        result = await handlePermissionsMethod(request, extension)
      } else if (request.method.startsWith('context.')) {
        result = await handleContextMethod(request)
      } else if (request.method.startsWith('search.')) {
        result = await handleSearchMethod(request, extension)
      } else {
        throw new Error(`Unknown method: ${request.method}`)
      }

      // Send success response
      sendResponse(iframeRef.value, {
        id: request.id,
        result,
      })
    } catch (error) {
      console.error('[HaexHub] Extension request error:', error)

      // Send error response
      sendResponse(iframeRef.value, {
        id: request.id,
        error: {
          code: 'INTERNAL_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error',
          details: error,
        },
      })
    }
  }

  const sendResponse = (
    iframe: HTMLIFrameElement,
    response: ExtensionResponse,
  ) => {
    iframe.contentWindow?.postMessage(response, '*')
  }

  // Register/unregister message listener
  onMounted(() => {
    window.addEventListener('message', handleMessage)
  })

  onUnmounted(() => {
    window.removeEventListener('message', handleMessage)
  })

  return {
    handleMessage,
  }
}

// ==========================================
// Extension Methods
// ==========================================

async function handleExtensionMethod(
  request: ExtensionRequest,
  extension: ComputedRef<IHaexHubExtensionLink | undefined>,
) {
  switch (request.method) {
    case 'extension.getInfo':
      return {
        keyHash: extension.value?.id || '', // TODO: Real key hash
        name: extension.value?.name || '',
        fullId: `${extension.value?.id}/${extension.value?.name}@${extension.value?.version}`,
        version: extension.value?.version || '',
        displayName: extension.value?.name,
        namespace: extension.value?.author,
        allowedOrigin: window.location.origin, // "tauri://localhost"
      }

    case 'extensions.getDependencies':
      // TODO: Implement dependencies from manifest
      return []

    default:
      throw new Error(`Unknown extension method: ${request.method}`)
  }
}

// ==========================================
// Database Methods
// ==========================================

async function handleDatabaseMethod(
  request: ExtensionRequest,
  extension: ComputedRef<IHaexHubExtensionLink | undefined>,
) {
  const { currentVault } = useVaultStore()
  if (!currentVault) {
    throw new Error('No vault available')
  }

  if (!extension.value) {
    throw new Error('Extension not found')
  }

  const params = request.params as { query?: string; params?: unknown[] }

  switch (request.method) {
    case 'db.query': {
      // Validate permission
      await validateDatabaseAccess(extension.value, params.query || '', 'read')

      // Execute query
      const result = await currentVault.drizzle.execute(params.query || '')

      return {
        rows: result.rows || [],
        rowsAffected: 0,
        lastInsertId: undefined,
      }
    }

    case 'db.execute': {
      // Validate permission
      await validateDatabaseAccess(extension.value, params.query || '', 'write')

      // Execute query
      const result = await currentVault.drizzle.execute(params.query || '')

      return {
        rows: [],
        rowsAffected: result.rowsAffected || 0,
        lastInsertId: result.lastInsertId,
      }
    }

    case 'db.transaction': {
      const statements =
        (request.params as { statements?: string[] }).statements || []

      // Validate all statements
      for (const stmt of statements) {
        await validateDatabaseAccess(extension.value, stmt, 'write')
      }

      // Execute transaction
      await currentVault.drizzle.transaction(async (tx) => {
        for (const stmt of statements) {
          await tx.execute(stmt)
        }
      })

      return { success: true }
    }

    default:
      throw new Error(`Unknown database method: ${request.method}`)
  }
}

// ==========================================
// Permission Validation
// ==========================================

async function validateDatabaseAccess(
  extension: IHaexHubExtensionLink,
  query: string,
  operation: 'read' | 'write',
): Promise<void> {
  // Extract table name from query
  const tableMatch = query.match(/(?:FROM|INTO|UPDATE|TABLE)\s+(\w+)/i)
  if (!tableMatch) {
    throw new Error('Could not extract table name from query')
  }

  const tableName = tableMatch[1]

  // Check if it's the extension's own table
  const extensionPrefix = `${extension.id}_${extension.name?.replace(/-/g, '_')}_`
  const isOwnTable = tableName.startsWith(extensionPrefix)

  if (isOwnTable) {
    // Own tables: always allowed
    return
  }

  // External table: Check permissions
  const hasPermission = await checkDatabasePermission(
    extension.id,
    tableName,
    operation,
  )

  if (!hasPermission) {
    throw new Error(`Permission denied: ${operation} access to ${tableName}`)
  }
}

async function checkDatabasePermission(
  extensionId: string,
  tableName: string,
  operation: 'read' | 'write',
): Promise<boolean> {
  // TODO: Query permissions from database
  // SELECT * FROM db_extension_permissions
  // WHERE extension_id = ? AND resource = ? AND operation = ?

  console.warn('TODO: Implement permission check', {
    extensionId,
    tableName,
    operation,
  })

  // For now: deny by default
  return false
}

// ==========================================
// Permission Methods
// ==========================================

async function handlePermissionsMethod(
  request: ExtensionRequest,
  extension: ComputedRef<IHaexHubExtensionLink | undefined>,
) {
  switch (request.method) {
    case 'permissions.database.request': {
      const params = request.params as {
        resource: string
        operation: 'read' | 'write'
        reason?: string
      }

      // TODO: Show user dialog to grant/deny permission
      console.log('[HaexHub] Permission request:', params)

      // For now: return ASK
      return {
        status: 'ask',
        permanent: false,
      }
    }

    case 'permissions.database.check': {
      const params = request.params as {
        resource: string
        operation: 'read' | 'write'
      }

      const hasPermission = await checkDatabasePermission(
        extension.value?.id || '',
        params.resource,
        params.operation,
      )

      return {
        status: hasPermission ? 'granted' : 'denied',
        permanent: true,
      }
    }

    default:
      throw new Error(`Unknown permission method: ${request.method}`)
  }
}

// ==========================================
// Context Methods
// ==========================================

async function handleContextMethod(request: ExtensionRequest) {
  const { theme } = useThemeStore()
  const { locale } = useI18n()

  switch (request.method) {
    case 'context.get':
      return {
        theme: theme.value || 'system',
        locale: locale.value,
        platform: detectPlatform(),
      }

    default:
      throw new Error(`Unknown context method: ${request.method}`)
  }
}

function detectPlatform(): 'desktop' | 'mobile' | 'tablet' {
  const width = window.innerWidth
  if (width < 768) return 'mobile'
  if (width < 1024) return 'tablet'
  return 'desktop'
}

// ==========================================
// Search Methods
// ==========================================

async function handleSearchMethod(
  request: ExtensionRequest,
  extension: ComputedRef<IHaexHubExtensionLink | undefined>,
) {
  switch (request.method) {
    case 'search.respond': {
      const params = request.params as {
        requestId: string
        results: unknown[]
      }

      // TODO: Store search results for display
      console.log('[HaexHub] Search results from extension:', params)

      return { success: true }
    }

    default:
      throw new Error(`Unknown search method: ${request.method}`)
  }
}

/**
 * Standardized error types for frontend error handling
 */

export interface AppError {
  code: string
  message: string
  details?: Record<string, unknown>
}

/**
 * Error codes for different error categories
 */
export const ErrorCodes = {
  UNKNOWN: 'UNKNOWN',
  IO_ERROR: 'IO_ERROR',
  COMMAND_ERROR: 'COMMAND_ERROR',
  HOTKEY_ERROR: 'HOTKEY_ERROR',
  VALIDATION_ERROR: 'VALIDATION_ERROR',
  PLATFORM_NOT_SUPPORTED: 'PLATFORM_NOT_SUPPORTED',
  NETWORK_ERROR: 'NETWORK_ERROR',
} as const

export type ErrorCode = typeof ErrorCodes[keyof typeof ErrorCodes]

/**
 * Parse unknown error into structured AppError
 */
export function parseError(error: unknown): AppError {
  if (error === null || error === undefined) {
    return { code: ErrorCodes.UNKNOWN, message: 'Unknown error occurred' }
  }

  if (typeof error === 'string') {
    return { code: ErrorCodes.UNKNOWN, message: error }
  }

  if (error instanceof Error) {
    return { code: ErrorCodes.UNKNOWN, message: error.message }
  }

  if (typeof error === 'object') {
    const obj = error as Record<string, unknown>

    // Handle Tauri IPC errors
    if ('message' in obj && typeof obj.message === 'string') {
      return {
        code: (obj.code as string) || ErrorCodes.UNKNOWN,
        message: obj.message,
        details: obj.details as Record<string, unknown> | undefined,
      }
    }
  }

  return { code: ErrorCodes.UNKNOWN, message: String(error) }
}

/**
 * Get user-friendly error message from unknown error
 */
export function getErrorMessage(error: unknown): string {
  return parseError(error).message
}

/**
 * Check if error is a specific error code
 */
export function isErrorCode(error: unknown, code: ErrorCode): boolean {
  return parseError(error).code === code
}

/**
 * Create a formatted error message for display
 */
export function formatError(error: unknown, prefix?: string): string {
  const parsed = parseError(error)
  const message = prefix ? `${prefix}: ${parsed.message}` : parsed.message
  return message
}

/**
 * Global Console Interceptor Plugin
 * Captures all console messages app-wide for debugging
 */

export interface ConsoleLog {
  timestamp: string
  level: 'log' | 'info' | 'warn' | 'error' | 'debug'
  message: string
}

// Global storage for console logs
export const globalConsoleLogs = ref<ConsoleLog[]>([])

// Store original console methods
const originalConsole = {
  log: console.log,
  info: console.info,
  warn: console.warn,
  error: console.error,
  debug: console.debug,
}

function interceptConsole(level: 'log' | 'info' | 'warn' | 'error' | 'debug') {
  console[level] = function (...args: unknown[]) {
    // Call original console method
    originalConsole[level].apply(console, args)

    // Add to global log display
    const timestamp = new Date().toLocaleTimeString()
    const message = args
      .map((arg) => {
        if (arg === null) return 'null'
        if (arg === undefined) return 'undefined'
        if (typeof arg === 'object') {
          try {
            return JSON.stringify(arg, null, 2)
          } catch {
            return String(arg)
          }
        }
        return String(arg)
      })
      .join(' ')

    globalConsoleLogs.value.push({
      timestamp,
      level,
      message,
    })

    // Limit to last 1000 logs
    if (globalConsoleLogs.value.length > 1000) {
      globalConsoleLogs.value = globalConsoleLogs.value.slice(-1000)
    }
  }
}

export default defineNuxtPlugin(() => {
  // Enable console interceptor
  interceptConsole('log')
  interceptConsole('info')
  interceptConsole('warn')
  interceptConsole('error')
  interceptConsole('debug')

  console.log('[HaexHub] Global console interceptor installed')

  return {
    provide: {
      consoleLogs: globalConsoleLogs,
      clearConsoleLogs: () => {
        globalConsoleLogs.value = []
      },
    },
  }
})

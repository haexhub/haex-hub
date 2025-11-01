export default defineNuxtPlugin({
  name: 'init-logger',
  enforce: 'pre',
  parallel: false,
  setup() {
    // Add global error handler for better debugging
    window.addEventListener('error', (event) => {
      console.error('[HaexHub] Global error caught:', {
        message: event.message,
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno,
        error: event.error,
        stack: event.error?.stack,
      })
    })

    window.addEventListener('unhandledrejection', (event) => {
      console.error('[HaexHub] Unhandled rejection:', {
        reason: event.reason,
        promise: event.promise,
      })
    })
  },
})

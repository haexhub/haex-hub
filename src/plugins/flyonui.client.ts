import 'flyonui/flyonui'

import type { HSOverlay, IStaticMethods } from 'flyonui/flyonui'
declare global {
  interface Window {
    HSStaticMethods: IStaticMethods
    HSOverlay: typeof HSOverlay
  }
}

export default defineNuxtPlugin(() => {
  const router = useRouter()
  router.afterEach(async () => {
    setTimeout(() => {
      if (window.HSStaticMethods) {
        window.HSStaticMethods.autoInit()
      }
    }, 50)
  })

  if (import.meta.client) {
    setTimeout(() => {
      if (window.HSStaticMethods) {
        window.HSStaticMethods.autoInit()
      }
    }, 50)
  }
})

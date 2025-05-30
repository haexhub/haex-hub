import 'flyonui/flyonui'

import type { HSOverlay, IStaticMethods, HSAccordion } from 'flyonui/flyonui'
declare global {
  interface Window {
    HSStaticMethods: IStaticMethods
    HSOverlay: typeof HSOverlay
    HSAccordion: typeof HSAccordion
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

  /* if (import.meta.client) {
    setTimeout(() => {
      if (window.HSStaticMethods) {
        window.HSStaticMethods.autoInit()
      }
    }, 50)
  } */
})

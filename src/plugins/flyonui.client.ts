import 'flyonui/flyonui'

import type {
  HSOverlay,
  IStaticMethods,
  HSAccordion,
  HSDropdown,
} from 'flyonui/flyonui'
declare global {
  interface Window {
    HSStaticMethods: IStaticMethods
    HSOverlay: typeof HSOverlay
    HSAccordion: typeof HSAccordion
    HSDropdown: typeof HSDropdown
  }
}

export default defineNuxtPlugin(() => {
  const router = useRouter()
  router.afterEach(async () => {
    setTimeout(() => {
      if (window.HSStaticMethods) {
        window.HSStaticMethods.autoInit()
      }
    }, 500)
  })

  if (import.meta.client) {
    setTimeout(() => {
      if (window.HSStaticMethods) {
        window.HSStaticMethods.autoInit()
      }
    }, 500)
  }
})

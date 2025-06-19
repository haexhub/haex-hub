import passwordActionMenuDe from '@/stores/passwords/actionMenu/de.json'
import passwordActionMenuEn from '@/stores/passwords/actionMenu/en.json'

export default defineI18nConfig(() => {
  return {
    legacy: false,
    messages: {
      de: {
        //passwordActionMenu: passwordActionMenuDe,
      },
      en: {
        //passwordActionMenu: passwordActionMenuEn,
      },
    },
  }
})

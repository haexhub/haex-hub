import type { IActionMenuItem } from '~/components/ui/button/types'
import de from './de.json'
import en from './en.json'

export const usePasswordsActionMenuStore = defineStore(
  'passwordsActionMenuStore',
  () => {
    const { t } =
      useI18n()
      /* {
      messages: {
        de: { passwordActionMenu: de },
        en: { passwordActionMenu: en },
      },
    } */

    const menu = computed<IActionMenuItem[]>(() => [
      {
        label: 'passwordActionMenu.group.create',
        icon: 'mdi:folder-plus-outline',
        to: {
          name: 'passwordGroupCreate',
          params: {
            ...useRouter().currentRoute.value.params,
            groupId: usePasswordGroupStore().currentGroupId,
          },
          query: {
            ...useRouter().currentRoute.value.query,
          },
        },
      },
      {
        label: 'passwordActionMenu.entry.create',
        icon: 'mdi:key-plus',
        to: {
          name: 'passwordItemCreate',
          params: {
            ...useRouter().currentRoute.value.params,
            groupId: usePasswordGroupStore().currentGroupId,
          },
          query: {
            ...useRouter().currentRoute.value.query,
          },
        },
      },
    ])

    return {
      menu,
    }
  },
)

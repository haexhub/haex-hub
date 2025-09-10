//import type { IActionMenuItem } from '~/components/ui/button/types'
import type { DropdownMenuItem } from '@nuxt/ui'
import de from './de.json'
import en from './en.json'

export const usePasswordsActionMenuStore = defineStore(
  'passwordsActionMenuStore',
  () => {
    const { $i18n } = useNuxtApp()

    $i18n.setLocaleMessage('de', {
      ...de,
    })
    $i18n.setLocaleMessage('en', { ...en })

    const localeRoute = useLocaleRoute()
    const menu = computed<DropdownMenuItem[]>(() => [
      {
        label: $i18n.t('group.create'),
        icon: 'mdi:folder-plus-outline',
        type: 'link',
        onSelect: () => {
          navigateTo(
            localeRoute({
              name: 'passwordGroupCreate',
              params: {
                ...useRouter().currentRoute.value.params,
                groupId: usePasswordGroupStore().currentGroupId,
              },
            }),
          )
        },
      },
      {
        label: $i18n.t('entry.create'),
        icon: 'mdi:key-plus',
        type: 'link',
        onSelect: () => {
          navigateTo(
            localeRoute({
              name: 'passwordItemCreate',
              params: {
                ...useRouter().currentRoute.value.params,
                groupId: usePasswordGroupStore().currentGroupId,
              },
            }),
          )
        },
      },
    ])

    return {
      menu,
    }
  },
)

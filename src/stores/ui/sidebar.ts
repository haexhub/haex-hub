import type { RouteLocationAsRelativeGeneric } from 'vue-router'

export interface ISidebarItem {
  name: string
  icon: string
  tooltip?: string
  id: string
  to?: RouteLocationAsRelativeGeneric
  iconType?: 'icon' | 'svg'
  onSelect?: () => void
}

export const useSidebarStore = defineStore('sidebarStore', () => {
  const isVisible = ref(true)

  const menu = ref<ISidebarItem[]>([
    {
      id: 'haex-pass',
      name: 'HaexPass',
      icon: 'mdi:safe',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
    {
      id: 'haex-extensions',
      name: 'Haex Extensions',
      icon: 'gg:extension',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'extensionOverview' }))
      },
    },
    {
      id: 'test',
      name: 'Test',
      icon: 'mdi:account',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
    {
      id: 'test2',
      name: 'Testsdfsfsdfsdf',
      icon: 'mdi:account',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
    {
      id: 'test3',
      name: 'Testsdfsfsdfsdf',
      icon: 'mdi:account',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
    {
      id: 'test4',
      name: 'Testsdfsfsdfsdf',
      icon: 'mdi:account',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
    {
      id: 'test5',
      name: 'Testsdfsfsdfsdf',
      icon: 'mdi:account',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
    {
      id: 'test6',
      name: 'Testsdfsfsdfsdf',
      icon: 'mdi:account',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
    {
      id: 'test7',
      name: 'Testsdfsfsdfsdf',
      icon: 'mdi:account',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
    {
      id: 'test8',
      name: 'Testsdfsfsdfsdf',
      icon: 'mdi:account',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
    {
      id: 'test9',
      name: 'Testsdfsfsdfsdf',
      icon: 'mdi:account',

      onSelect: () => {
        navigateTo(useLocalePath()({ name: 'passwordGroupItems' }))
      },
    },
  ])

  return {
    menu,
    isVisible,
  }
})

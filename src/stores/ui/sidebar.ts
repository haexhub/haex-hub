import { getSingleRouteParam } from '~/composables/helper';
import type { RouteLocationRaw } from 'vue-router';

export interface ISidebarItem {
  name: string;
  icon: string;
  tooltip?: string;
  id: string;
  type: 'browser' | 'extension';
}

export const useSidebarStore = defineStore('sidebarStore', () => {
  const menu = ref<ISidebarItem[]>([
    {
      id: 'haex-browser',
      name: 'Haex Browser',
      icon: 'solar:global-outline',
      type: 'browser',
    },

    {
      id: 'haex-vault',
      name: 'Haex Vault',
      icon: 'gg:extension',
      type: 'extension',
    },
  ]);

  /* const loadAsync = async (id: string) => {
    extensions.value.some(async (extension) => {
      if (extension.id === id) {
        await navigateTo(
          useLocalePath()({ name: 'extension', params: { extensionId: id } })
        );
      } else {
      }
    });
  }; */

  return {
    menu,
    //loadAsync,
  };
});

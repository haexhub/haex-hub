import { getSingleRouteParam } from '~/composables/helper';
import type { RouteLocationRaw } from 'vue-router';

export interface IExtensionLink {
  name: string;
  icon: string;
  tooltip?: string;
  id: string;
}

export const useExtensionsStore = defineStore('extensionsStore', () => {
  const extensions = ref<IExtensionLink[]>([
    {
      id: 'haex-browser',
      name: 'Haex Browser',
      icon: 'solar:global-outline',
    },

    {
      id: 'extensions',
      name: 'sidebar.extensions',
      icon: 'gg:extension',
    },

    {
      id: 'settings',
      name: 'sidebar.settings',
      icon: 'ph:gear-six',
    },
  ]);

  const currentRoute = useRouter().currentRoute.value;

  const isActive = (id: string) =>
    computed(
      () =>
        currentRoute.name === 'extension' &&
        currentRoute.params.extensionId === id
    );

  const loadAsync = async (id: string) => {
    extensions.value.some(async (extension) => {
      if (extension.id === id) {
        await navigateTo(
          useLocalePath()({ name: 'extension', params: { extensionId: id } })
        );
      } else {
      }
    });
  };

  return {
    extensions,
    loadAsync,
    isActive,
  };
});

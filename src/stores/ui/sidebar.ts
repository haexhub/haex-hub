import { getSingleRouteParam } from "~/composables/helper";
import type { RouteLocationRaw, RouteLocationAsRelativeGeneric } from "vue-router";

export interface ISidebarItem {
    name: string;
    icon: string;
    tooltip?: string;
    id: string;
    to?: RouteLocationAsRelativeGeneric;
}

export const useSidebarStore = defineStore("sidebarStore", () => {
    const menu = ref<ISidebarItem[]>([
        {
            id: "haex-browser",
            name: "Haex Browser",
            icon: "solar:global-outline",
            to: { name: "haexBrowser" },
        },

        {
            id: "haex-extensions-add",
            name: "Haex Extensions",
            icon: "gg:extension",
            to: { name: "haexExtensionAdd" },
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

import type { RouteLocationAsRelativeGeneric } from "vue-router";

export interface ISidebarItem {
  name: string;
  icon: string;
  tooltip?: string;
  id: string;
  to?: RouteLocationAsRelativeGeneric;
  iconType?: "icon" | "svg";
}

export const useSidebarStore = defineStore("sidebarStore", () => {
  const isVisible = ref(true);

  const menu = ref<ISidebarItem[]>([
    {
      id: "haex-extensions-add",
      name: "Haex Extensions",
      icon: "gg:extension",
      to: { name: "extensionOverview" },
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
    isVisible,
    //loadAsync,
  };
});

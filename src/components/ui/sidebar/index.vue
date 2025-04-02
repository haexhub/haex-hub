<template>
  <aside
    class="flex shrink-0 transition-[width] ease-in duration-300 z-30 h-full overflow-hidden fixed sm:relative left-0 shadow border-r border-base-300"
  >
    <div class="sm:flex flex-col w-14 bg-base-200 shrink-0 h-full hidden">
      <img
        src="/logo.svg"
        class="bg-primary p-3 size-16"
      />

      <div class="flex flex-col justify-between h-full overflow-y-scroll z-10">
        <div class="flex flex-col space-y-2 text-base-content/90">
          <template v-for="item in menu.top">
            <UiSidebarLink
              v-if="item.to"
              :to="item.to ?? ''"
              :icon="item.icon"
              :label="$t(item.label)"
            />

            <UiSidebarButton
              v-else
              :icon="item.icon"
              :label="$t(item.label)"
              @click="item.click"
            />
          </template>
        </div>

        <div class="flex flex-col space-y-2 text-base-content/90">
          <template v-for="item in menu.bottom">
            <UiSidebarLink
              v-if="item.to"
              :to="item.to ?? ''"
              :icon="item.icon"
              :label="$t(item.label)"
            />

            <UiSidebarButton
              v-else
              :icon="item.icon"
              :label="$t(item.label)"
              @click="item.click"
            />
          </template>
          <!-- <UiSidebarLink
            v-for="item in menu.bottom"
            :icon="item.icon"
            :to="item.to ?? ''"
            :label="item.label"
            @click="item.click"
          /> -->
        </div>
      </div>
    </div>

    <!-- <div class="bg-base-100 flex flex-col w-full overflow-clip">
      <div
        class="h-16 flex items-center sm:justify-center justify-end md:justify-start bg-base-300 shrink-0"
      >
        <button
          class="top-3 left-2 absolute z-30 duration-1000 btn btn-square btn-primary transition-opacity btn-outline sm:hidden"
          @click="show = !show"
        >
          <Icon
            name="mdi:menu"
            size="28"
          />
        </button>
        <span
          class="px-4 font-semibold text-base-content shrink-0 sm:bg-transparent bg-primary h-full flex items-center rounded-l-lg"
        >
          <p>Haex Vault</p>
        </span>

        <img
          src="/logo.svg"
          class="bg-primary p-3 size-16 shrink-0 sm:hidden rounded-r-lg"
        />

        <button
          class="btn btn-square btn-primary btn-outline mr-2 ml-auto hidden sm:flex"
          @click="show = false"
        >
          <Icon
            name="mdi:close"
            size="28"
          />
        </button>
      </div>

      <div class="overflow-scroll flex pb-4 relative">
        <slot />
      </div>
    </div> -->
  </aside>
</template>

<script lang="ts" setup>
defineProps({
  menu: {
    type: Object as PropType<ISidebarMenu>,
    default: () => {},
  },
});

//const show = ref(true);
const { show } = storeToRefs(useSidebarStore());
</script>

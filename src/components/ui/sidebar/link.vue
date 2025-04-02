<template>
  <li
    @click="triggerNavigate"
    class="hover:text-primary"
  >
    <UiTooltip
      :tooltip="tooltip ?? name"
      direction="right-end"
    >
      <NuxtLinkLocale
        :class="{ ['bg-base-300']: isActive }"
        :to="{
          name: type === 'browser' ? 'haexBrowser' : 'haexExtension',
          params: type === 'browser' ? {} : { extensionId: id },
        }"
        class="flex items-center justify-center cursor-pointer tooltip-toogle"
        ref="link"
      >
        <Icon
          :name="icon"
          class="shrink-0 size-6"
        />
      </NuxtLinkLocale>
    </UiTooltip>
  </li>
</template>

<script setup lang="ts">
import { type ISidebarItem } from '#imports';

const props = defineProps<ISidebarItem>();

const router = useRouter();

const isActive = computed(() => {
  if (props.type === 'browser') {
    return router.currentRoute.value.name === 'haexBrowser';
  } else if (props.type === 'extension') {
    return (
      router.currentRoute.value.name === 'haexExtension' &&
      getSingleRouteParam(router.currentRoute.value.params.extensionId) ===
        props.id
    );
  }
});

const link = useTemplateRef('link');

const triggerNavigate = () => link.value?.$el.click();

/* computed(() => {
  const found = useRouter()
    .getRoutes()
    .find((route) => route.name === useLocaleRoute()(props.to)?.name);
  console.log('found route', found, useRoute());
  return (
    found?.name === useRoute().name ||
    found?.children.some((child) => child.name === useRoute().name)
  );
}); */
</script>

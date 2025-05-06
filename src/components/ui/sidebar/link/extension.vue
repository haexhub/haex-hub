<template>
  <li
    @click="triggerNavigate"
    class="hover:text-primary rounded"
    :class="{ ['bg-base-300']: isActive }"
  >
    <UiTooltip :tooltip="tooltip ?? name" direction="right-start">
      <NuxtLinkLocale
        :to="{ name: 'haexExtension', params: { extensionId: props.id } }"
        class="flex items-center justify-center cursor-pointer tooltip-toogle"
        ref="link"
      >
        <div v-html="icon" class="shrink-0 size-6" />
        <!-- <Icon mode="svg" :name="icon" class="shrink-0 size-6" /> -->
      </NuxtLinkLocale>
    </UiTooltip>
  </li>
</template>

<script setup lang="ts">
import { type ISidebarItem } from "#imports";

const props = defineProps<ISidebarItem>();
console.log("image", props.icon);
const router = useRouter();

const isActive = computed(() => {
  if (props.to?.name === "haexExtension") {
    return getSingleRouteParam(router.currentRoute.value.params.extensionId) === props.id;
  } else {
    return props.to?.name === router.currentRoute.value.meta.name;
  }
});

const link = useTemplateRef("link");

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

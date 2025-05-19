<template>
  <li
    @click="triggerNavigate"
    class="hover:text-primary rounded"
    :class="{ ['bg-base-200 text-base-content']: isActive }"
  >
    <UiTooltip :tooltip="tooltip ?? name" direction="right-start">
      <NuxtLinkLocale
        :to
        class="flex items-center justify-center cursor-pointer tooltip-toogle"
        ref="linkRef"
      >
        <div v-if="iconType === 'svg'" v-html="icon" class="shrink-0 size-5" />
        <Icon v-else :name="icon" size="1.5em" />
      </NuxtLinkLocale>
    </UiTooltip>
  </li>
</template>

<script setup lang="ts">
import { type ISidebarItem } from '#imports'

const props = defineProps<ISidebarItem>()

const router = useRouter()

console.log('to', props.to)
const isActive = computed(() => {
  if (props.to?.name === 'haexExtension') {
    return (
      getSingleRouteParam(router.currentRoute.value.params.extensionId) ===
      props.id
    )
  } else {
    return props.to?.name === router.currentRoute.value.meta.name
  }
})

const linkRef = useTemplateRef('linkRef')

const triggerNavigate = () => linkRef.value?.$el.click()

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

<template>
  <li
    class="rounded"
    :class="{
      ['bg-base-300/35 ']: isActive,
    }"
    @click="triggerNavigate"
  >
    <UiTooltip
      :tooltip="tooltip ?? name"
      direction="right-start"
    >
      <NuxtLinkLocale
        ref="linkRef"
        :to
        class="flex items-center justify-center cursor-pointer tooltip-toogle"
      >
        <div
          v-if="iconType === 'svg'"
          class="shrink-0 size-5"
          v-html="icon"
        />
        <Icon
          v-else
          :name="icon"
          size="1.5em"
        />
      </NuxtLinkLocale>
    </UiTooltip>
  </li>
</template>

<script setup lang="ts">
import type { ISidebarItem } from '#imports'

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
    return (
      props.to?.name === router.currentRoute.value.meta.name ||
      router
        .getRoutes()
        .find((route) => route.meta.name === props.to?.name)
        ?.children.some(
          (route) => route.meta?.name === router.currentRoute.value.meta.name,
        )
    )
  }
})

const linkRef = useTemplateRef('linkRef')

const triggerNavigate = () => linkRef.value?.$el.click()
</script>

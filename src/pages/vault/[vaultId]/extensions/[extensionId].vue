<template>
  <div class="w-full h-full overflow-scroll">
    <div>
      {{ iframeIndex }}
    </div>
    <iframe
      v-if="iframeIndex"
      ref="iFrameRef"
      class="w-full h-full"
      :src="iframeIndex"
      sandbox="allow-scripts allow-same-origin"
      allow="autoplay; speaker-selection; encrypted-media;"
    />
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'haexExtension',
})

const { extensionEntry: iframeSrc } = storeToRefs(useExtensionsStore())

const iframeIndex = computed(() =>
  iframeSrc.value ? `${iframeSrc.value}/index.html` : '',
)
</script>

<i18n lang="yaml">
de:
  loading: Erweiterung wird geladen
en:
  loading: Extension is loading
</i18n>

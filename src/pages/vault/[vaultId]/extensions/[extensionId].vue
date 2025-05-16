<template>
  <div class="w-full h-full overflow-scroll">
    <!-- <div>
      {{ iframeSrc }}
    </div> -->
    <iframe
      v-if="iframeSrc"
      class="w-full h-full"
      @load=""
      ref="iFrameRef"
      :src="iframeIndex"
      sandbox="allow-scripts allow-same-origin"
      allow="autoplay; speaker-selection; encrypted-media;"
    >
    </iframe>
    <!--  <p v-else>{{ t("loading") }}</p> -->
    <audio controls :src="audioTest">
      Dein Browser unterstützt das Audio-Element nicht.
    </audio>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'haexExtension',
})

const { t } = useI18n()
const iframeRef = useTemplateRef('iFrameRef')
const { extensionEntry: iframeSrc, currentExtension } = storeToRefs(
  useExtensionsStore()
)
const audioTest = computed(() => `${iframeSrc.value}/sounds/music/demo.mp3`)
watch(audioTest, () => console.log('audioTest', audioTest.value), {
  immediate: true,
})

const iframeIndex = computed(() => `${iframeSrc.value}/index.html`)
const extensionStore = useExtensionsStore()

watch(iframeSrc, () => console.log('iframeSrc', iframeSrc.value), {
  immediate: true,
})

onMounted(async () => {
  /* const minfest = await extensionStore.readManifestFileAsync(
    currentExtension.value!.id,
    currentExtension.value!.version
  );
  console.log("manifest", minfest, extensionStore.extensionEntry); */
})
</script>

<i18n lang="yaml">
de:
  loading: Erweiterung wird geladen
en:
  loading: Extension is loading
</i18n>

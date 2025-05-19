<template>
  <div class="w-full h-full overflow-scroll">
    <!-- <div>
      {{ iframeSrc }}
    </div> -->
    <iframe
      v-if="iframeIndex"
      class="w-full h-full"
      @load=""
      ref="iFrameRef"
      :src="iframeIndex"
      sandbox="allow-scripts allow-same-origin"
      allow="autoplay; speaker-selection; encrypted-media;"
    >
    </iframe>

    <UiButton @click="go = true">Go</UiButton>
    <!--  <p v-else>{{ t("loading") }}</p> -->
    {{ audioTest }}
    <audio v-if="go" controls :src="audioTest">
      Dein Browser unterstützt das Audio-Element nicht.
    </audio>

    <video v-if="go" controls width="600" :src="demoVideo"></video>
    <div v-if="audioError">
      Fehler beim Laden der Audio-Datei: {{ audioError }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { appDataDir, join, resourceDir } from '@tauri-apps/api/path'

definePageMeta({
  name: 'haexExtension',
})

const { t } = useI18n()
const iframeRef = useTemplateRef('iFrameRef')
const { extensionEntry: iframeSrc, currentExtension } = storeToRefs(
  useExtensionsStore()
)
const audioAssetUrl = ref('')
const audioError = ref('')
const audioTest = convertFileSrc(
  await join(await appDataDir(), 'resources/demo.mp3')
)

//computed(() => `${iframeSrc.value}/sounds/music/demo.mp3`)

const go = ref(false)
const iframeIndex = computed(() => `${iframeSrc.value}/index.html`)
const demoVideo = computed(() => `${iframeSrc.value}/sounds/music/demo.mp3`)

const extensionStore = useExtensionsStore()

watch(
  demoVideo,
  async () => {
    const res = await fetch(
      '/home/haex/.local/share/space.haex.hub/extensions/pokedemo/1.0/sounds/music/demo.mp3'
    )
    console.log('respo', res)

    console.log('iframeSrc', iframeSrc.value)
  },
  {
    immediate: true,
  }
)
</script>

<i18n lang="yaml">
de:
  loading: Erweiterung wird geladen
en:
  loading: Extension is loading
</i18n>

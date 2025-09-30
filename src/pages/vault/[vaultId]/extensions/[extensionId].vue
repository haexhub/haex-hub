<template>
  <div class="w-full h-full overflow-scroll">
    <div
      v-if="!iFrameSrc"
      class="flex items-center justify-center h-full"
    >
      <p>{{ t('loading') }}</p>
    </div>
    <iframe
      v-else
      ref="iFrameRef"
      class="w-full h-full"
      :src="iFrameSrc"
      sandbox="allow-scripts "
      allow="autoplay; speaker-selection; encrypted-media;"
    />
  </div>
</template>

<script setup lang="ts">
import { useExtensionMessageHandler } from '~/composables/extensionMessageHandler'

definePageMeta({
  name: 'haexExtension',
})

const { t } = useI18n()

const iFrameRef = useTemplateRef('iFrameRef')

const { extensionEntry: iframeSrc, currentExtension } =
  storeToRefs(useExtensionsStore())

const iFrameSrc = computed(() =>
  iframeSrc.value ? `${iframeSrc.value}/index.html` : '',
)

useExtensionMessageHandler(iFrameRef, currentExtension)

const { currentTheme } = storeToRefs(useUiStore())
const { locale } = useI18n()

watch([currentTheme, locale], () => {
  if (iFrameRef.value?.contentWindow) {
    iFrameRef.value.contentWindow.postMessage(
      {
        type: 'context.changed',
        data: {
          context: {
            theme: currentTheme.value || 'system',
            locale: locale.value,
            platform:
              window.innerWidth < 768
                ? 'mobile'
                : window.innerWidth < 1024
                  ? 'tablet'
                  : 'desktop',
          },
        },
        timestamp: Date.now(),
      },
      '*',
    )
  }
})
</script>

<i18n lang="yaml">
de:
  loading: Erweiterung wird geladen
en:
  loading: Extension is loading
</i18n>

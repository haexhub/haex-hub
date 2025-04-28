<template>
  <div class="w-full h-full">
    <iframe class="w-full h-full" @load="" ref="iFrameRef"> </iframe>
    <p>{{ t("loading") }}</p>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: "haexExtension",
});

const iframeRef = useTemplateRef("iFrameRef");
const { t } = useI18n();
</script>

<i18n lang="yaml">
de:
  loading: Erweiterung wird geladen
en:
  loading: Extension is loading
</i18n>

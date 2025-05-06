<template>
  {{ iframeSrc }}
  <div class="w-full h-full">
    <iframe
      v-if="iframeSrc"
      class="w-full h-full"
      @load=""
      ref="iFrameRef"
      :src="iframeSrc"
      sandbox="allow-scripts allow-same-origin"
    >
    </iframe>
    <p v-else>{{ t("loading") }}</p>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: "haexExtension",
});

const { t } = useI18n();
const iframeRef = useTemplateRef("iFrameRef");

const { extensionEntry: iframeSrc, currentExtension } = storeToRefs(useExtensionsStore());
const extensionStore = useExtensionsStore();

watch(iframeSrc, () => console.log("iframeSrc", iframeSrc.value), { immediate: true });

onMounted(async () => {
  const minfest = await extensionStore.readManifestFileAsync(
    currentExtension.value!.id,
    currentExtension.value!.version
  );
  console.log("manifest", minfest, extensionStore.extensionEntry);
});
</script>

<i18n lang="yaml">
de:
  loading: Erweiterung wird geladen
en:
  loading: Extension is loading
</i18n>

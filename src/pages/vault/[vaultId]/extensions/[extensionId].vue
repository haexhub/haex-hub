<template>
  <div class="h-full flex flex-col">
    <!-- Tab Bar -->
    <div class="flex gap-2 p-2 bg-default overflow-x-auto border-b">
      <div
        v-for="tab in tabsStore.sortedTabs"
        :key="tab.extension.id"
        :class="[
          'btn btn-sm gap-2',
          tabsStore.activeTabId === tab.extension.id
            ? 'btn-primary'
            : 'btn-ghost',
        ]"
        @click="tabsStore.setActiveTab(tab.extension.id)"
      >
        {{ tab.extension.name }}
        <button
          class="ml-1 hover:text-error"
          @click.stop="tabsStore.closeTab(tab.extension.id)"
        >
          <Icon
            name="mdi:close"
            size="16"
          />
        </button>
      </div>
    </div>

    <!-- IFrame Container -->
    <div class="flex-1 relative overflow-hidden">
      <div
        v-for="tab in tabsStore.sortedTabs"
        :key="tab.extension.id"
        :style="{ display: tab.isVisible ? 'block' : 'none' }"
        class="w-full h-full"
      >
        <iframe
          :ref="
            (el) => registerIFrame(tab.extension.id, el as HTMLIFrameElement)
          "
          class="w-full h-full"
          :src="getExtensionUrl(tab.extension)"
          sandbox="allow-scripts"
          allow="autoplay; speaker-selection; encrypted-media;"
        />
      </div>

      <!-- Loading State -->
      <div
        v-if="tabsStore.tabCount === 0"
        class="flex items-center justify-center h-full"
      >
        <p>{{ t('loading') }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useExtensionMessageHandler } from '~/composables/extensionMessageHandler'
import { useExtensionTabsStore } from '~/stores/extensions/tabs'
import type { IHaexHubExtension } from '~/types/haexhub'

definePageMeta({
  name: 'haexExtension',
})

const { t } = useI18n()

const tabsStore = useExtensionTabsStore()

// Extension aus Route öffnen
//const extensionId = computed(() => route.params.extensionId as string)

const { currentExtensionId } = storeToRefs(useExtensionsStore())
watchEffect(() => {
  if (currentExtensionId.value) {
    tabsStore.openTab(currentExtensionId.value)
  }
})

const messageHandlers = new Map<string, boolean>()

watch(
  () => tabsStore.openTabs,
  (tabs) => {
    tabs.forEach((tab, id) => {
      if (tab.iframe && !messageHandlers.has(id)) {
        const iframeRef = ref(tab.iframe)
        const extensionRef = computed(() => tab.extension)
        useExtensionMessageHandler(iframeRef, extensionRef)
        messageHandlers.set(id, true)
      }
    })
  },
  { deep: true },
)

// IFrame Registrierung und Message Handler Setup
/* const iframeRefs = new Map<string, HTMLIFrameElement>()
const setupMessageHandlers = new Set<string>() */

const registerIFrame = (extensionId: string, el: HTMLIFrameElement | null) => {
  if (!el) return
  tabsStore.registerIFrame(extensionId, el)
}
// Extension URL generieren
const getExtensionUrl = (extension: IHaexHubExtension) => {
  const info = { id: extension.id, version: extension.version }
  const jsonString = JSON.stringify(info)
  const bytes = new TextEncoder().encode(jsonString)
  const encoded = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')

  const url = `haex-extension://${encoded}/index.html`
  console.log('Extension URL:', url, 'for', extension.name)
  return url
}

// Context Changes an alle Tabs broadcasten
const { currentTheme } = storeToRefs(useUiStore())
const { locale } = useI18n()

watch([currentTheme, locale], () => {
  tabsStore.broadcastToAllTabs({
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
  })
})

// Cleanup beim Verlassen
onBeforeUnmount(() => {
  // Optional: Alle Tabs schließen oder offen lassen
  // tabsStore.closeAllTabs()
})
</script>

<i18n lang="yaml">
de:
  loading: Erweiterung wird geladen
en:
  loading: Extension is loading
</i18n>

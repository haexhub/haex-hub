<template>
  <div class="h-screen w-screen flex flex-col">
    <!-- Tab Bar -->
    <div
      class="flex gap-2 bg-base-200 overflow-x-auto border-b border-base-300 flex-shrink-0"
    >
      <UButton
        v-for="tab in tabsStore.sortedTabs"
        :key="tab.extension.id"
        :class="[
          'gap-2',
          tabsStore.activeTabId === tab.extension.id ? 'primary' : 'neutral',
        ]"
        @click="tabsStore.setActiveTab(tab.extension.id)"
      >
        {{ tab.extension.name }}

        <template #trailing>
          <div
            class="ml-1 hover:text-error"
            @click.stop="tabsStore.closeTab(tab.extension.id)"
          >
            <Icon
              name="mdi:close"
              size="16"
            />
          </div>
        </template>
      </UButton>
    </div>

    <!-- IFrame Container -->
    <div class="flex-1 relative min-h-0">
      <div
        v-for="tab in tabsStore.sortedTabs"
        :key="tab.extension.id"
        :style="{ display: tab.isVisible ? 'block' : 'none' }"
        class="absolute inset-0"
      >
        <iframe
          :ref="
            (el) => registerIFrame(tab.extension.id, el as HTMLIFrameElement)
          "
          class="w-full h-full border-0"
          :src="getExtensionUrl(tab.extension)"
          sandbox="allow-scripts allow-storage-access-by-user-activation allow-forms"
          allow="autoplay; speaker-selection; encrypted-media;"
        />
      </div>

      <!-- Loading State -->
      <div
        v-if="tabsStore.tabCount === 0"
        class="absolute inset-0 flex items-center justify-center"
      >
        <p>{{ t('loading') }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  useExtensionMessageHandler,
  registerExtensionIFrame,
  unregisterExtensionIFrame,
} from '~/composables/extensionMessageHandler'
import { useExtensionTabsStore } from '~/stores/extensions/tabs'
import type { IHaexHubExtension } from '~/types/haexhub'
import { platform } from '@tauri-apps/plugin-os'
import { EXTENSION_PROTOCOL_NAME, EXTENSION_PROTOCOL_PREFIX } from '~/config/constants'

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

// Setup global message handler EINMAL im Setup-Kontext
// Dies registriert den globalen Event Listener
const dummyIframeRef = ref<HTMLIFrameElement | null>(null)
const dummyExtensionRef = computed(() => null)
useExtensionMessageHandler(dummyIframeRef, dummyExtensionRef)

const registerIFrame = (extensionId: string, el: HTMLIFrameElement | null) => {
  if (!el) return

  // Registriere IFrame im Store
  tabsStore.registerIFrame(extensionId, el)

  // Registriere IFrame im globalen Message Handler Registry
  const tab = tabsStore.openTabs.get(extensionId)
  if (tab?.extension) {
    registerExtensionIFrame(el, tab.extension)
  }
}

// Cleanup wenn Tabs geschlossen werden
watch(
  () => tabsStore.openTabs,
  (newTabs, oldTabs) => {
    if (oldTabs) {
      // Finde gelöschte Tabs
      oldTabs.forEach((tab, id) => {
        if (!newTabs.has(id) && tab.iframe) {
          unregisterExtensionIFrame(tab.iframe)
        }
      })
    }
  },
  { deep: true },
)
const os = await platform()

// Extension URL generieren
const getExtensionUrl = (extension: IHaexHubExtension) => {
  // Extract key_hash from full_extension_id (everything before first underscore)
  const firstUnderscoreIndex = extension.id.indexOf('_')
  if (firstUnderscoreIndex === -1) {
    console.error('Invalid full_extension_id format:', extension.id)
    return ''
  }

  const keyHash = extension.id.substring(0, firstUnderscoreIndex)

  const info = {
    key_hash: keyHash,
    name: extension.name,
    version: extension.version,
  }

  const jsonString = JSON.stringify(info)
  const bytes = new TextEncoder().encode(jsonString)
  const encodedInfo = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')

  // 'android', 'ios', 'windows' etc.
  let schemeUrl: string

  if (os === 'android' || os === 'windows') {
    // Android/Windows: http://<scheme>.localhost/path
    schemeUrl = `http://${EXTENSION_PROTOCOL_NAME}.localhost/${encodedInfo}/index.html`
  } else {
    // macOS/Linux/iOS: Klassisch scheme://localhost/path
    schemeUrl = `${EXTENSION_PROTOCOL_PREFIX}localhost/${encodedInfo}/index.html`
  }

  return schemeUrl
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

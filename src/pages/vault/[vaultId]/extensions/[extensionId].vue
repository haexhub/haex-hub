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

      <!-- Console Tab -->
      <UButton
        :class="['gap-2', showConsole ? 'primary' : 'neutral']"
        @click="showConsole = !showConsole"
      >
        <Icon
          name="mdi:console"
          size="16"
        />
        Console
        <UBadge
          v-if="visibleLogs.length > 0"
          size="xs"
          color="primary"
        >
          {{ visibleLogs.length }}
        </UBadge>
      </UButton>
    </div>

    <!-- IFrame Container -->
    <div class="flex-1 relative min-h-0">
      <!-- Extension IFrames -->
      <div
        v-for="tab in tabsStore.sortedTabs"
        :key="tab.extension.id"
        :style="{ display: tab.isVisible && !showConsole ? 'block' : 'none' }"
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

      <!-- Console View -->
      <div
        v-if="showConsole"
        class="absolute inset-0 bg-base-100 flex flex-col"
      >
        <!-- Console Header -->
        <div
          class="p-2 border-b border-base-300 flex justify-between items-center"
        >
          <h3 class="font-semibold">Console Output</h3>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            @click="$clearConsoleLogs()"
          >
            Clear
          </UButton>
        </div>

        <!-- Console Logs -->
        <div class="flex-1 overflow-y-auto p-2 font-mono text-sm">
          <!-- Info banner if logs are limited -->
          <div
            v-if="consoleLogs.length > maxVisibleLogs"
            class="mb-2 p-2 bg-warning/10 border border-warning/30 rounded text-xs"
          >
            Showing last {{ maxVisibleLogs }} of {{ consoleLogs.length }} logs
          </div>

          <!-- Simple log list instead of accordion for better performance -->
          <div
            v-if="visibleLogs.length > 0"
            class="space-y-1"
          >
            <div
              v-for="(log, index) in visibleLogs"
              :key="index"
              class="border-b border-base-200 pb-2"
            >
              <!-- Log header with timestamp and level -->
              <div class="flex justify-between items-center mb-1">
                <span class="text-xs opacity-60">
                  [{{ log.timestamp }}] [{{ log.level.toUpperCase() }}]
                </span>
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-heroicons-clipboard-document"
                  @click="copyToClipboard(log.message)"
                />
              </div>
              <!-- Log message -->
              <pre
                :class="[
                  'text-xs whitespace-pre-wrap break-all',
                  log.level === 'error' ? 'text-error' : '',
                  log.level === 'warn' ? 'text-warning' : '',
                  log.level === 'info' ? 'text-info' : '',
                  log.level === 'debug' ? 'text-base-content/70' : '',
                ]"
              >{{ log.message }}</pre>
            </div>
          </div>

          <div
            v-if="visibleLogs.length === 0"
            class="text-center text-base-content/50 py-8"
          >
            No console messages yet
          </div>
        </div>
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
import {
  EXTENSION_PROTOCOL_NAME,
  EXTENSION_PROTOCOL_PREFIX,
} from '~/config/constants'

definePageMeta({
  name: 'haexExtension',
})

const { t } = useI18n()

const tabsStore = useExtensionTabsStore()

// Console logging - use global logs from plugin
const { $consoleLogs, $clearConsoleLogs } = useNuxtApp()
const showConsole = ref(false)
const maxVisibleLogs = ref(100) // Limit for performance on mobile
const consoleLogs = $consoleLogs as Ref<
  Array<{
    timestamp: string
    level: 'log' | 'info' | 'warn' | 'error' | 'debug'
    message: string
  }>
>

// Only show last N logs for performance
const visibleLogs = computed(() => {
  return consoleLogs.value.slice(-maxVisibleLogs.value)
})

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

// Track which iframes have been registered to prevent duplicate registrations
const registeredIFrames = new WeakSet<HTMLIFrameElement>()

const registerIFrame = (extensionId: string, el: HTMLIFrameElement | null) => {
  if (!el) return

  // Prevent duplicate registration (Vue calls ref functions on every render)
  if (registeredIFrames.has(el)) {
    return
  }

  console.log('[Vue Debug] ========== registerIFrame called ==========')
  console.log('[Vue Debug] Extension ID:', extensionId)
  console.log('[Vue Debug] Element:', 'HTMLIFrameElement')

  // Mark as registered
  registeredIFrames.add(el)

  // Registriere IFrame im Store
  tabsStore.registerIFrame(extensionId, el)

  // Registriere IFrame im globalen Message Handler Registry
  const tab = tabsStore.openTabs.get(extensionId)
  if (tab?.extension) {
    console.log('[Vue Debug] Registering iframe in message handler for:', tab.extension.name)
    registerExtensionIFrame(el, tab.extension)
    console.log('[Vue Debug] Registration complete!')
  } else {
    console.error('[Vue Debug] ❌ No tab found for extension ID:', extensionId)
  }
  console.log('[Vue Debug] ========================================')
}

// Listen for console messages from extensions (via postMessage)
const handleExtensionConsole = (event: MessageEvent) => {
  if (event.data?.type === 'console.forward') {
    const { timestamp, level, message } = event.data.data
    consoleLogs.value.push({
      timestamp,
      level,
      message: `[Extension] ${message}`,
    })

    // Limit to last 1000 logs
    if (consoleLogs.value.length > 1000) {
      consoleLogs.value = consoleLogs.value.slice(-1000)
    }
  }
}

onMounted(() => {
  window.addEventListener('message', handleExtensionConsole)
})

onBeforeUnmount(() => {
  window.removeEventListener('message', handleExtensionConsole)

  // Unregister all iframes when the page unmounts
  tabsStore.openTabs.forEach((tab) => {
    if (tab.iframe) {
      unregisterExtensionIFrame(tab.iframe)
    }
  })
})

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

// Copy to clipboard function
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    // Optional: Show success toast
    console.log('Copied to clipboard')
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}
</script>

<i18n lang="yaml">
de:
  loading: Erweiterung wird geladen
en:
  loading: Extension is loading
</i18n>

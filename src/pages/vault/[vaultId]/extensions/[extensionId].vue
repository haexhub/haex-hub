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
        <!-- Error overlay for dev extensions when server is not reachable -->
        <div
          v-if="tab.extension.devServerUrl && iframe.errors[tab.extension.id]"
          class="absolute inset-0 bg-base-100 flex items-center justify-center p-8"
        >
          <div class="max-w-md space-y-4 text-center">
            <Icon
              name="mdi:alert-circle-outline"
              size="64"
              class="mx-auto text-warning"
            />
            <h3 class="text-lg font-semibold">
              {{ t('devServer.notReachable.title') }}
            </h3>
            <p class="text-sm opacity-70">
              {{
                t('devServer.notReachable.description', {
                  url: tab.extension.devServerUrl,
                })
              }}
            </p>
            <div class="bg-base-200 p-4 rounded text-left text-xs font-mono">
              <p class="opacity-70 mb-2">
                {{ t('devServer.notReachable.howToStart') }}
              </p>
              <code class="block">cd /path/to/extension</code>
              <code class="block">npm run dev</code>
            </div>
            <UButton
              :label="t('devServer.notReachable.retry')"
              @click="retryLoadIFrame(tab.extension.id)"
            />
          </div>
        </div>

        <iframe
          :ref="
            (el) => registerIFrame(tab.extension.id, el as HTMLIFrameElement)
          "
          class="w-full h-full border-0"
          :src="
            getExtensionUrl(
              tab.extension.publicKey,
              tab.extension.name,
              tab.extension.version,
              'index.html',
              tab.extension.devServerUrl ?? undefined,
            )
          "
          :sandbox="iframe.sandboxAttributes(tab.extension.devServerUrl)"
          allow="autoplay; speaker-selection; encrypted-media;"
          @error="onIFrameError(tab.extension.id)"
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
                >{{ log.message }}</pre
              >
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
  EXTENSION_PROTOCOL_PREFIX,
  EXTENSION_PROTOCOL_NAME,
} from '~/config/constants'

definePageMeta({
  name: 'extension',
})

const { t } = useI18n()

const tabsStore = useExtensionTabsStore()

// Track iframe errors (for dev mode)
//const iframeErrors = ref<Record<string, boolean>>({})

const sandboxDefault = [
  'allow-scripts',
  'allow-storage-access-by-user-activation',
  'allow-forms',
] as const

const iframe = reactive<{
  errors: Record<string, boolean>
  sandboxAttributes: (devUrl?: string | null) => string
}>({
  errors: {},
  sandboxAttributes: (devUrl) => {
    return devUrl
      ? [...sandboxDefault, 'allow-same-origin'].join(' ')
      : sandboxDefault.join(' ')
  },
})

const { platform } = useDeviceStore()

// Generate extension URL (uses cached platform)
const getExtensionUrl = (
  publicKey: string,
  name: string,
  version: string,
  assetPath: string = 'index.html',
  devServerUrl?: string,
) => {
  if (!publicKey || !name || !version) {
    console.error('Missing required extension fields')
    return ''
  }

  // If dev server URL is provided, load directly from dev server
  if (devServerUrl) {
    const cleanUrl = devServerUrl.replace(/\/$/, '')
    const cleanPath = assetPath.replace(/^\//, '')
    return cleanPath ? `${cleanUrl}/${cleanPath}` : cleanUrl
  }

  const extensionInfo = {
    name,
    publicKey,
    version,
  }
  const encodedInfo = btoa(JSON.stringify(extensionInfo))

  if (platform === 'android' || platform === 'windows') {
    // Android: Tauri uses http://{scheme}.localhost format
    return `http://${EXTENSION_PROTOCOL_NAME}.localhost/${encodedInfo}/${assetPath}`
  } else {
    // Desktop: Use custom protocol with base64 as host
    return `${EXTENSION_PROTOCOL_PREFIX}${encodedInfo}/${assetPath}`
  }
}

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
    console.log(
      '[Vue Debug] Registering iframe in message handler for:',
      tab.extension.name,
    )
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

// Handle iframe errors (e.g., dev server not running)
const onIFrameError = (extensionId: string) => {
  iframe.errors[extensionId] = true
}

// Retry loading iframe (clears error and reloads)
const retryLoadIFrame = (extensionId: string) => {
  iframe.errors[extensionId] = false
  // Reload the iframe by updating the tab
  const tab = tabsStore.openTabs.get(extensionId)
  if (tab?.iframe) {
    tab.iframe.src = tab.iframe.src // Force reload
  }
}
</script>

<i18n lang="yaml">
de:
  loading: Erweiterung wird geladen
  devServer:
    notReachable:
      title: Dev-Server nicht erreichbar
      description: Der Dev-Server unter {url} ist nicht erreichbar.
      howToStart: 'So starten Sie den Dev-Server:'
      retry: Erneut versuchen
en:
  loading: Extension is loading
  devServer:
    notReachable:
      title: Dev Server Not Reachable
      description: The dev server at {url} is not reachable.
      howToStart: 'To start the dev server:'
      retry: Retry
</i18n>

<template>
  <div class="browser">
    <div class="browser-controls">
      <button
        @click="$emit('goBack', activeTabId)"
        :disabled="!activeTabId"
      >
        ←
      </button>
      <button
        @click="$emit('goForward', activeTabId)"
        :disabled="!activeTabId"
      >
        →
      </button>
      <button @click="$emit('createTab')">+</button>

      <HaexBrowserUrlBar
        :url="activeTab?.url || ''"
        :isLoading="activeTab?.isLoading || false"
        @submit="handleUrlSubmit"
      />
    </div>

    <HaexBrowserTabBar
      :tabs="tabs"
      :activeTabId="activeTabId"
      @closeTab="$emit('closeTab', $event)"
      @activateTab="$emit('activateTab', $event)"
    />

    <div
      class="browser-content"
      ref="contentRef"
    >
      <!-- Die eigentlichen Webview-Inhalte werden von Tauri verwaltet -->
      <div
        v-if="!activeTabId"
        class="empty-state"
      >
        <p>
          Kein Tab geöffnet. Erstellen Sie einen neuen Tab mit dem + Button.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { Window } from '@tauri-apps/api/window';
import { getCurrentWebview, Webview } from '@tauri-apps/api/webview';
/* const appWindow = new Window('uniqueLabel');
const webview = new Webview(appWindow, 'theUniqueLabel', {
  url: 'https://www.google.de',
  x: 0,
  y: 0,
  height: 1000,
  width: 1000,
});

webview.once('tauri://created', function () {
  console.log('create new webview');
}); */

interface Tab {
  id: string;
  title: string;
  url: string;
  isLoading: boolean;
  isActive: boolean;
  window_label: string;
}

interface Props {
  tabs: Tab[];
  activeTabId: string | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'createTab'): void;
  (e: 'closeTab', tabId: string): void;
  (e: 'navigate', tabId: string, url: string): void;
  (e: 'goBack', tabId: string | null): void;
  (e: 'goForward', tabId: string | null): void;
  (e: 'activateTab', tabId: string | null): void;
}>();

const { initializeAsync, processNavigation, injectContentScripts } =
  useBrowserExtensionStore();
const contentRef = ref<HTMLDivElement | null>(null);
//const extensionManager = ref<ExtensionManager>(new ExtensionManager());

const activeTab = computed(() =>
  props.tabs?.find((tab) => tab.id === props.activeTabId)
);

onMounted(async () => {
  // Initialisiere das Erweiterungssystem
  await initializeAsync();
  // Aktualisiere die Webview-Größe
  await updateWebviewBoundsAsync();
  //window.addEventListener('resize', updateWebviewBounds);
});

// Wenn ein neuer Tab aktiviert wird, injiziere Content-Scripts
/* watch(
  () => props.activeTabId,
  async (newTabId) => {
    if (newTabId && props.tabs.length > 0) {
      const activeTab = props.tabs.find((tab) => tab.id === newTabId);
      if (activeTab) {
        // Warte kurz, bis die Seite geladen ist
        setTimeout(() => {
          injectContentScripts(activeTab.window_label);
        }, 500);

        // Aktualisiere die Webview-Größe
        updateWebviewBounds();
      }
    }
  }
); */

const handleUrlSubmit = (url: string) => {
  if (props.activeTabId) {
    // Prüfe URL mit Erweiterungen vor der Navigation
    if (processNavigation(url)) {
      emit('navigate', props.activeTabId, url);
    } else {
      console.log('Navigation blockiert durch Erweiterung');
      // Hier könnten Sie eine Benachrichtigung anzeigen
    }
  }
};

const updateWebviewBoundsAsync = async () => {
  if (!contentRef.value) return;

  const rect = contentRef.value.getBoundingClientRect();
  const bounds = {
    x: rect.left,
    y: rect.top,
    width: rect.width,
    height: rect.height,
  };

  /* await invoke('update_window_bounds', {
    contentBounds: { x: bounds.x, y: bounds.y },
    contentSize: { width: bounds.width, height: bounds.height },
  }); */
};
</script>

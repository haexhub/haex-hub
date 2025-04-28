<template>
    <div>
        browser {{ useRouter().currentRoute.value.meta.name }}
        <HaexBrowser
            :tabs="tabs"
            :activeTabId="activeTabId"
            @createTab="createNewTab"
            @closeTab="closeTab"
            @navigate="navigateToUrl"
            @goBack="goBack"
            @goForward="goForward"
        />
    </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Window, getCurrentWindow } from "@tauri-apps/api/window";
import { Webview } from "@tauri-apps/api/webview";

definePageMeta({
    name: "haexBrowser",
});

interface Tab {
    id: string;
    title: string;
    url: string;
    isLoading: boolean;
    isActive: boolean;
    window_label: string;
}

const tabs = ref<Tab[]>([]);
const activeTabId = ref<string | null>(null);

let unlistenTabCreated: UnlistenFn | null = null;
let unlistenTabClosed: UnlistenFn | null = null;

onMounted(async () => {
    // Erstelle einen ersten Tab beim Start
    //createNewTab("https://www.google.com");

    // Höre auf Tab-Events
    unlistenTabCreated = await listen("tab-created", (event) => {
        const newTab = event.payload as Tab;

        tabs.value = tabs.value.map((tab) => ({
            ...tab,
            isActive: tab.id === newTab.id,
        }));

        if (!tabs.value.some((tab) => tab.id === newTab.id)) {
            tabs.value.push(newTab);
        }

        activeTabId.value = newTab.id;
    });

    unlistenTabClosed = await listen("tab-closed", (event) => {
        const closedTabId = event.payload as string;
        tabs.value = tabs.value.filter((tab) => tab.id !== closedTabId);
    });
});

onUnmounted(() => {
    if (unlistenTabCreated) unlistenTabCreated();
    if (unlistenTabClosed) unlistenTabClosed();
});

const createNewTab = async (url: string = "about:blank") => {
    try {
        /* const appWindow = new Window('uniqueLabel111', {
      fullscreen: true,
    });
 */
        /* const appWindow = getCurrentWindow();

    const webview = new Webview(appWindow, 'theUniqueLabel', {
      url: 'https://github.com/tauri-apps/tauri',
      height: 1000,
      width: 1000,
      x: 110,
      y: 0,
    });
    await webview.show(); */
        //console.log('create webview', webview);
        const tab_id = "foo";
        await invoke("create_tab", { url, tabId: "foo" });
    } catch (error) {
        console.error("Fehler beim Erstellen des Tabs:", error);
    }
};

const closeTab = async (tabId: string) => {
    try {
        //await invoke('close_tab', { tabId });
    } catch (error) {
        console.error("Fehler beim Schließen des Tabs:", error);
    }
};

const navigateToUrl = async (tabId: string, url: string) => {
    try {
        //await invoke('navigate_to_url', { tabId, url });
    } catch (error) {
        console.error("Fehler bei der Navigation:", error);
    }
};

const goBack = async (tabId: string | null) => {
    try {
        //await invoke('go_back', { tabId });
    } catch (error) {
        console.error("Fehler beim Zurückgehen:", error);
    }
};

const goForward = async (tabId: string | null) => {
    try {
        //await invoke('go_forward', { tabId });
    } catch (error) {
        console.error("Fehler beim Vorwärtsgehen:", error);
    }
};
</script>

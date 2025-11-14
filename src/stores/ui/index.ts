import { breakpointsTailwind } from '@vueuse/core'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { HAEXTENSION_EVENTS } from '@haexhub/sdk'
import { broadcastContextToAllExtensions } from '~/composables/extensionMessageHandler'

import de from './de.json'
import en from './en.json'

export const useUiStore = defineStore('uiStore', () => {
  const breakpoints = useBreakpoints(breakpointsTailwind)

  // "smAndDown" gilt für sm, xs usw.
  const isSmallScreen = breakpoints.smaller('sm')

  const { $i18n } = useNuxtApp()
  const { locale } = useI18n({
    useScope: 'global',
  })

  $i18n.setLocaleMessage('de', {
    ui: de,
  })
  $i18n.setLocaleMessage('en', { ui: en })

  const availableThemes = ref([
    {
      value: 'dark',
      label: $i18n.t('ui.dark'),
      icon: 'line-md:moon-rising-alt-loop',
    },
    {
      value: 'light',
      label: $i18n.t('ui.light'),
      icon: 'line-md:moon-to-sunny-outline-loop-transition',
    },
    /*     {
      value: 'soft',
      label: t('ui.soft'),
      icon: 'line-md:paint-drop',
    },

    {
      value: 'corporate',
      label: t('ui.corporate'),
      icon: 'hugeicons:corporate',
    }, */
  ])

  const defaultTheme = ref('dark')

  const currentThemeName = ref(defaultTheme.value)

  const currentTheme = computed(
    () =>
      availableThemes.value.find(
        (theme) => theme.value === currentThemeName.value,
      ) ?? availableThemes.value.at(0),
  )

  const colorMode = useColorMode()

  watchImmediate(currentThemeName, () => {
    colorMode.preference = currentThemeName.value
  })

  // Broadcast theme and locale changes to extensions (including initial state)
  watch([currentThemeName, locale], async () => {
    const deviceStore = useDeviceStore()
    const platformValue = await deviceStore.platform
    const context = {
      theme: currentThemeName.value,
      locale: locale.value,
      platform: platformValue,
    }

    // Broadcast to iframe extensions (existing)
    broadcastContextToAllExtensions(context)

    // Update Tauri state and emit event for webview extensions
    try {
      await invoke('webview_extension_context_set', { context })
      console.log('[UI Store] Context set in Tauri state:', context)
      // Emit Tauri event so webview extensions can listen for changes
      await emit(HAEXTENSION_EVENTS.CONTEXT_CHANGED, { context })
      console.log('[UI Store] Emitted context change event:', context)
    } catch (error) {
      // Ignore error if not running in Tauri (e.g., browser mode)
      console.debug('[UI Store] Failed to update Tauri context:', error)
    }
  }, { immediate: true })

  const viewportHeightWithoutHeader = ref(0)
  const headerHeight = ref(0)

  return {
    availableThemes,
    viewportHeightWithoutHeader,
    headerHeight,
    currentTheme,
    currentThemeName,
    defaultTheme,
    isSmallScreen,
  }
})

import { breakpointsTailwind, useBreakpoints } from '@vueuse/core';
import de from './de.json';
import en from './en.json';

export interface ITheme {
  value: string,
  name: string,
  icon: string
}

export const useUiStore = defineStore('uiStore', () => {
  const breakpoints = useBreakpoints(breakpointsTailwind);

  const currentScreenSize = computed(() =>
    breakpoints.active().value.length > 0 ? breakpoints.active().value : 'xs'
  );

  const { t } = useI18n({
    messages: {
      de: { ui: de },
      en: { ui: en },
    },
  });

  const availableThemes = ref([
    {
      value: 'dark',
      name: t('ui.dark'),
      icon: 'line-md:moon-rising-alt-loop',
    },
    {
      value: 'light',
      name: t('ui.light'),
      icon: 'line-md:moon-to-sunny-outline-loop-transition',
    },
    { value: 'soft', name: t('ui.soft'), icon: 'line-md:paint-drop' },
    {
      value: 'corporate',
      name: t('ui.corporate'),
      icon: 'hugeicons:corporate',
    },
  ]);

  const defaultTheme = ref(availableThemes.value[0])

  const currentTheme = ref(defaultTheme);

  return {
    availableThemes,
    breakpoints,
    currentScreenSize,
    currentTheme,
    defaultTheme,
  };
});

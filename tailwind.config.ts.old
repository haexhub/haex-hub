import { defineConfig } from '@nuxtjs/tailwindcss/config';
//import { iconsPlugin, dynamicIconsPlugin } from '@egoist/tailwindcss-icons';
//import colors from 'tailwindcss/colors';
import themes from 'flyonui/src/theming/themes';
//import * as tailwindMotion from 'tailwindcss-motion';
import { addDynamicIconSelectors } from '@iconify/tailwind';

export default defineConfig({
  content: ['./src/**/*.{vue,ts,svg}', './node_modules/flyonui/dist/js/*.js'],
  darkMode: 'selector',
  plugins: [
    /* iconsPlugin(),
    dynamicIconsPlugin(), */
    addDynamicIconSelectors(),
    require('flyonui'),
    require('flyonui/plugin'),
    //tailwindMotion,
  ],

  flyonui: {
    themes: [
      {
        light: {
          ...themes.light,
          /* primary: colors.teal[500],
          secondary: colors.purple[500], */
        },
        soft: {
          ...themes.soft,
          /* primary: colors.teal[500],
          secondary: colors.purple[500], */
        },
        dark: {
          ...themes.dark,
          /* primary: colors.cyan[700], //colors.teal[600],
          secondary: colors.purple[500], */
          /* 'primary-content': '#000516',
          'secondary': '#008f9c',
          'secondary-content': '#000709',
          'accent': '#007f7a',
          'accent-content': '#d3e5e3',
          'neutral': '#321a15',
          'neutral-content': '#d3ccca',
          'base-100': '#002732',
          'base-200': '#00202a',
          'base-300': '#001a22',
          'base-content': '#c8cfd2',
          'info': '#0086b2',
          'info-content': '#00060c',
          'success': '#a5da00',
          'success-content': '#0a1100',
          'warning': '#ff8d00',
          'warning-content': '#160700',
          'error': '#c83849',
          'error-content': '#f9d9d9', */
        },
      },
    ],
    /*  themes: [
      {
        dark: {
          'primary': colors.teal[500],
          'primary-content': '#010811',
          'secondary': colors.purple[500],
          'secondary-content': '#130201',
          'accent': '#9b59b6',
          'accent-content': '#ebddf1',
          'neutral': '#95a5a6',
          'neutral-content': '#080a0a',
          'base-100': colors.slate[100],
          'base-200': colors.slate[400],
          'base-300': colors.slate[900],
          'base-content': colors.slate[800],
          'info': '#1abc9c',
          'info-content': '#000d09',
          'success': '#2ecc71',
          'success-content': '#010f04',
          'warning': '#f1c40f',
          'warning-content': '#140e00',
          'error': '#e74c3c',
          'error-content': '#130201',
        },

        light: {
          'primary': colors.teal[500],
          'primary-content': '#010811',
          'secondary': colors.purple[500],
          'secondary-content': '#130201',
          'accent': '#9b59b6',
          'accent-content': '#ebddf1',
          'neutral': '#95a5a6',
          'neutral-content': '#080a0a',
          'base-100': '#ecf0f1',
          'base-200': '#cdd1d2',
          'base-300': '#afb2b3',
          'base-content': '#131414',
          'info': '#1abc9c',
          'info-content': '#000d09',
          'success': '#2ecc71',
          'success-content': '#010f04',
          'warning': '#f1c40f',
          'warning-content': '#140e00',
          'error': '#e74c3c',
          'error-content': '#130201',
        },
      },
    ], */
  },

  /* theme: {
    extend: {
      colors: {
        'primary-active': colors.teal[600],
        'primary-focus': colors.teal[400],
        'primary-hover': colors.teal[400],
        'primary': colors.teal[500],

        'dark': {
          'primary-active': colors.teal[700],
          'primary-focus': colors.teal[600],
          'primary-hover': colors.teal[600],
          'primary': colors.teal[500],
        },
        'secondary': colors.sky[500],
      },

      fontFamily: {
        sans: [
          'Adelle',
          'Roboto Slab',
          'DejaVu Serif',
          'Georgia',
          'Graphik',
          'sans-serif',
        ],
        serif: ['Merriweather', 'serif'],
      },

      screens: {
        xs: '360px',
      },

      transitionProperty: {
        height: 'height',
      },
    },
  }, */
}); // satisfies Config;

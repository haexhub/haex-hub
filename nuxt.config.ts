import tailwindcss from "@tailwindcss/vite";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2024-11-01",

  modules: [
    "nuxt-zod-i18n",
    "@nuxtjs/i18n",
    "@pinia/nuxt",
    "@vueuse/nuxt",
    "@nuxt/icon",
    "nuxt-snackbar",
    "nuxt-svgo-loader",
  ],

  imports: {
    dirs: ["composables/**", "stores/**", "components/**", "pages/**", "types/**"],
  },

  i18n: {
    strategy: "prefix_and_default",
    defaultLocale: "de",
    vueI18n: "~/i18n/i18n.config.ts",

    locales: [
      { code: "de", language: "de-DE", isCatchallLocale: true },
      { code: "en", language: "en-EN" },
    ],

    detectBrowserLanguage: {
      useCookie: true,
      cookieKey: "i18n_redirected",
      redirectOn: "root", // recommended
    },
    types: "composition",
    bundle: {
      optimizeTranslationDirective: false,
    },
  },

  zodI18n: {
    localeCodesMapping: {
      "en-GB": "en",
      "de-DE": "de",
    },
  },

  runtimeConfig: {
    public: {
      haexVault: {
        lastVaultFileName: "lastVaults.json",
        //defaultDatabase: 'src/database/default.db',
      },
    },
  },

  css: ["~/assets/css/main.css"],

  devtools: { enabled: true },

  srcDir: "./src",
  // Enable SSG
  ssr: false,
  // Enables the development server to be discoverable by other devices when running on iOS physical devices
  devServer: { host: process.env.TAURI_DEV_HOST || "localhost", port: 3003 },

  vite: {
    plugins: [tailwindcss()],
    // Better support for Tauri CLI output
    clearScreen: false,
    // Enable environment variables
    // Additional environment variables can be found at
    // https://v2.tauri.app/reference/environment-variables/
    envPrefix: ["VITE_", "TAURI_"],
    server: {
      // Tauri requires a consistent port
      strictPort: true,
    },
  },
});

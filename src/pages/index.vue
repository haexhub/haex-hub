<template>
  <div class="items-center justify-center min-h-full flex w-full relative">
    <div class="fixed top-2 right-2">
      <UiDropdownLocale @select="setLocale" />
    </div>
    <div class="flex flex-col justify-center items-center gap-5 max-w-3xl">
      <!-- <img
        src="/logo.svg"
        class="bg-primary p-3 size-16 rounded-full"
        alt="HaexVault Logo"
      /> -->

      <span
        class="flex flex-wrap font-bold text-pretty text-xl gap-2 justify-center"
      >
        <p class="whitespace-nowrap">
          {{ t('welcome') }}
        </p>
        <UiTextGradient>Haex Hub</UiTextGradient>
      </span>

      <div class="flex flex-col md:flex-row gap-4 w-full h-24 md:h-auto">
        <VaultButtonCreate />

        <VaultButtonOpen
          v-model:open="passwordPromptOpen"
          :path="vaultPath"
        />
      </div>

      <div
        v-show="lastVaults.length"
        class="w-full"
      >
        <div class="font-thin text-sm justify-start px-2 pb-1">
          {{ t('lastUsed') }}
        </div>

        <div
          class="relative border-base-content/25 divide-base-content/25 flex w-full flex-col divide-y rounded-md border first:*:rounded-t-md last:*:rounded-b-md overflow-scroll"
        >
          <div
            v-for="vault in lastVaults"
            :key="vault.path"
            class="flex items-center justify-between group h-12 overflow-x-scroll"
          >
            <button
              class="link link-accent flex items-center no-underline justify-between text-nowrap text-sm md:text-base shrink w-full py-2 px-4"
              @click=";(passwordPromptOpen = true), (vaultPath = vault.path)"
            >
              <span class="block md:hidden">
                {{ vault.name }}
              </span>
              <span class="hidden md:block">
                {{ vault.path }}
              </span>
            </button>
            <button
              class="absolute right-2 btn btn-square btn-error btn-xs hidden group-hover:flex min-w-6"
            >
              <Icon
                name="mdi:trash-can-outline"
                @click="removeVaultAsync(vault.path)"
              />
            </button>
          </div>
        </div>
      </div>

      <div class="flex flex-col items-center gap-2">
        <h4>{{ t('sponsors') }}</h4>
        <div>
          <button @click="openUrl('https://itemis.com')">
            <UiLogoItemis class="text-[#00457C]" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'

definePageMeta({
  name: 'vaultOpen',
})

const passwordPromptOpen = ref(false)
const vaultPath = ref('')

const { t, setLocale } = useI18n()

const { syncLastVaultsAsync, removeVaultAsync } = useLastVaultStore()
const { lastVaults } = storeToRefs(useLastVaultStore())

await syncLastVaultsAsync()
</script>

<i18n lang="json">
{
  "de": {
    "welcome": "Viel Spass mit",
    "lastUsed": "Zuletzt verwendete Vaults",
    "sponsors": "Powered by"
  },
  "en": {
    "welcome": "Have fun with",
    "lastUsed": "Last used Vaults",
    "sponsors": "Powered by"
  }
}
</i18n>

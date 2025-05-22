<template>
  <div class="items-center justify-center min-h-full flex w-full relative">
    <div class="fixed top-2 right-2">
      <UiDropdownLocale @select="setLocale" />
    </div>
    <div class="flex flex-col justify-center items-center gap-5 max-w-3xl">
      <img
        src="/logo.svg"
        class="bg-primary p-3 size-16 rounded-full"
        alt="HaexVault Logo"
      />

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
          v-model:isOpen="passwordPromptOpen"
          :path="vaultPath"
        />
        <UiDialogTest />

        <UiDialog> aaaaaa </UiDialog>
        <button
          type="button"
          class="btn btn-primary"
          aria-haspopup="dialog"
          aria-expanded="false"
          aria-controls="basic-modal"
          data-overlay="#basic-modal1"
        >
          Open modal
        </button>

        <div
          id="basic-modal1"
          class="overlay modal overlay-open:opacity-100 hidden overlay-open:duration-300"
          role="dialog"
          tabindex="-1"
        >
          <div
            class="modal-dialog overlay-open:opacity-100 overlay-open:duration-300"
          >
            <div class="modal-content">
              <div class="modal-header">
                <h3 class="modal-title">Dialog Title</h3>
                <button
                  type="button"
                  class="btn btn-text btn-circle btn-sm absolute end-3 top-3"
                  aria-label="Close"
                  data-overlay="#basic-modal1"
                >
                  <span class="icon-[tabler--x] size-4"></span>
                </button>
              </div>
              <div class="modal-body">
                This is some placeholder content to show the scrolling behavior
                for modals. Instead of repeating the text in the modal, we use
                an inline style to set a minimum height, thereby extending the
                length of the overall modal and demonstrating the overflow
                scrolling. When content becomes longer than the height of the
                viewport, scrolling will move the modal as needed.
              </div>
              <div class="modal-footer">
                <button
                  type="button"
                  class="btn btn-soft btn-secondary"
                  data-overlay="#basic-modal1"
                >
                  Close
                </button>
                <button type="button" class="btn btn-primary">
                  Save changes
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-show="lastVaults.length" class="w-full">
        <div class="font-thin text-sm justify-start px-2 pb-1">
          {{ t('lastUsed') }}
        </div>

        <div
          class="relative border-base-content/25 divide-base-content/25 flex w-full flex-col divide-y rounded-md border first:*:rounded-t-md last:*:rounded-b-md overflow-scroll"
        >
          <div
            class="flex items-center justify-between group h-12 overflow-x-scroll"
            v-for="vault in lastVaults"
            :key="vault.path"
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

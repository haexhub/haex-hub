<template>
  <div class="items-center justify-center flex w-full h-full relative">
    <div class="absolute top-8 right-8 sm:top-4 sm:right-4">
      <UiDropdownLocale @select="onSelectLocale" />
    </div>

    <div class="flex flex-col justify-center items-center gap-5 max-w-3xl">
      <UiLogoHaexhub class="bg-primary p-3 size-16 rounded-full shrink-0" />
      <span
        class="flex flex-wrap font-bold text-pretty text-xl gap-2 justify-center"
      >
        <p class="whitespace-nowrap">
          {{ t('welcome') }}
        </p>
        <UiTextGradient>Haex Hub</UiTextGradient>
      </span>

      <div class="flex flex-col md:flex-row gap-4 w-full h-24 md:h-auto">
        <HaexVaultCreate />

        <HaexVaultOpen
          v-model:open="passwordPromptOpen"
          :path="selectedVault?.path"
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
          class="relative border-base-content/25 divide-base-content/25 flex w-full flex-col divide-y rounded-md border overflow-scroll"
        >
          <div
            v-for="vault in lastVaults"
            :key="vault.name"
            class="flex items-center justify-between group overflow-x-scroll"
          >
            <UButton
              variant="ghost"
              color="neutral"
              class="flex items-center no-underline justify-between text-nowrap text-sm md:text-base shrink w-full px-3"
              @click="
                () => {
                  passwordPromptOpen = true
                  selectedVault = vault
                }
              "
            >
              <span class="block">
                {{ vault.name }}
              </span>
            </UButton>
            <UButton
              color="error"
              square
              class="absolute right-2 hidden group-hover:flex min-w-6"
            >
              <Icon
                name="mdi:trash-can-outline"
                @click="removeVaultAsync(vault.name)"
              />
            </UButton>
          </div>
        </div>
      </div>

      <div class="flex flex-col items-center gap-2">
        <h4>{{ t('sponsors') }}</h4>
        <div>
          <UButton
            variant="link"
            @click="openUrl('https://itemis.com')"
          >
            <UiLogoItemis class="text-[#00457C]" />
          </UButton>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import type { Locale } from 'vue-i18n'

definePageMeta({
  name: 'vaultOpen',
})
const { t, setLocale } = useI18n()

const passwordPromptOpen = ref(false)
const selectedVault = ref<IVaultInfo>()

const { syncLastVaultsAsync, removeVaultAsync } = useLastVaultStore()
const { lastVaults } = storeToRefs(useLastVaultStore())

onMounted(async () => {
  await syncLastVaultsAsync()
})

const onSelectLocale = async (locale: Locale) => {
  await setLocale(locale)
}
</script>

<i18n lang="yaml">
de:
  welcome: 'Viel Spass mit'
  lastUsed: 'Zuletzt verwendete Vaults'
  sponsors: 'Supported by'

en:
  welcome: 'Have fun with'
  lastUsed: 'Last used Vaults'
  sponsors: 'Supported by'
</i18n>

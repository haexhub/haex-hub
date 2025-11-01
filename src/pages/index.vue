<template>
  <div class="h-full">
    <NuxtLayout>
      <div
        class="flex flex-col justify-center items-center gap-5 mx-auto h-full overflow-scroll"
      >
        <UiLogoHaexhub class="bg-primary p-3 size-16 rounded-full shrink-0" />
        <span
          class="flex flex-wrap font-bold text-pretty text-xl gap-2 justify-center"
        >
          <p class="whitespace-nowrap">
            {{ t('welcome') }}
          </p>
          <UiTextGradient>Haex Hub</UiTextGradient>
        </span>

        <div class="flex flex-col gap-4 h-24 items-stretch justify-center">
          <HaexVaultCreate />

          <HaexVaultOpen
            v-model:open="passwordPromptOpen"
            :path="selectedVault?.path"
          />
        </div>

        <div
          v-show="lastVaults.length"
          class="max-w-md w-full sm:px-5"
        >
          <div class="font-thin text-sm pb-1 w-full">
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
              <UiButtonContext
                variant="ghost"
                color="neutral"
                size="xl"
                class="flex items-center no-underline justify-between text-nowrap text-sm md:text-base shrink w-full hover:bg-default"
                :context-menu-items="[
                  {
                    icon: 'mdi:trash-can-outline',
                    label: t('remove.button'),
                    onSelect: () => prepareRemoveVault(vault.name),
                    color: 'error',
                  },
                ]"
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
              </UiButtonContext>
              <UButton
                color="error"
                square
                class="absolute right-2 hidden group-hover:flex min-w-6"
              >
                <Icon
                  name="mdi:trash-can-outline"
                  @click="prepareRemoveVault(vault.name)"
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

      <UiDialogConfirm
        v-model:open="showRemoveDialog"
        :title="t('remove.title')"
        :description="t('remove.description', { vaultName: vaultToBeRemoved })"
        @confirm="onConfirmRemoveAsync"
      />
    </NuxtLayout>
  </div>
</template>

<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'

import type { VaultInfo } from '@bindings/VaultInfo'

definePageMeta({
  name: 'vaultOpen',
})

const { t } = useI18n()

const passwordPromptOpen = ref(false)
const selectedVault = ref<VaultInfo>()

const showRemoveDialog = ref(false)

const { lastVaults } = storeToRefs(useLastVaultStore())

const { syncLastVaultsAsync, moveVaultToTrashAsync } = useLastVaultStore()
const { syncDeviceIdAsync } = useDeviceStore()

const vaultToBeRemoved = ref('')
const prepareRemoveVault = (vaultName: string) => {
  vaultToBeRemoved.value = vaultName
  showRemoveDialog.value = true
}

const toast = useToast()
const onConfirmRemoveAsync = async () => {
  try {
    await moveVaultToTrashAsync(vaultToBeRemoved.value)
    showRemoveDialog.value = false
    await syncLastVaultsAsync()
  } catch (error) {
    toast.add({
      color: 'error',
      description: JSON.stringify(error),
    })
  }
}

onMounted(async () => {
  try {
    await syncLastVaultsAsync()
    await syncDeviceIdAsync()
  } catch (error) {
    console.error('ERROR: ', error)
  }
})
</script>

<i18n lang="yaml">
de:
  welcome: 'Viel Spass mit'
  lastUsed: 'Zuletzt verwendete Vaults'
  sponsors: Supported by
  remove:
    button: Löschen
    title: Vault löschen
    description: Möchtest du die Vault {vaultName} wirklich löschen?

en:
  welcome: 'Have fun with'
  lastUsed: 'Last used Vaults'
  sponsors: 'Supported by'
  remove:
    button: Delete
    title: Delete Vault
    description: Are you sure you really want to delete {vaultName}?
</i18n>

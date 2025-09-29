<template>
  <UiDialogConfirm
    v-model:open="open"
    :confirm-label="t('open')"
    :description="vault.path || path"
    @confirm="onOpenDatabase"
  >
    <!-- <UiButton
      :label="t('vault.open')"
      :ui="{
        base: 'px-3 py-2',
      }"
      icon="mdi:folder-open-outline"
      size="xl"
      variant="outline"
      block
      @click.stop="onLoadDatabase"
    /> -->

    <template #title>
      <i18n-t
        keypath="title"
        tag="p"
        class="flex gap-x-2 text-wrap"
      >
        <template #haexvault>
          <UiTextGradient>HaexVault</UiTextGradient>
        </template>
      </i18n-t>
    </template>

    <template #body>
      <UForm
        :state="vault"
        class="flex flex-col gap-4 w-full h-full justify-center"
      >
        <UiInputPassword
          v-model="vault.password"
          class="w-full"
          autofocus
        />

        <UButton
          hidden
          type="submit"
          @click="onOpenDatabase"
        />
      </UForm>
    </template>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
/* import { open as openVault } from '@tauri-apps/plugin-dialog' */
import { vaultSchema } from './schema'

const open = defineModel<boolean>('open', { default: false })
const props = defineProps<{
  path?: string
}>()

const { t } = useI18n()

const vault = reactive<{
  name: string
  password: string
  path: string | null
  type: 'password' | 'text'
}>({
  name: '',
  password: '',
  path: '',
  type: 'password',
})

/* const onLoadDatabase = async () => {
  try {
    vault.path = await openVault({
      multiple: false,
      directory: false,
      filters: [
        {
          name: 'HaexVault',
          extensions: ['db'],
        },
      ],
    })

    console.log('onLoadDatabase', vault.path)
    if (!vault.path) {
      open.value = false
      return
    }

    open.value = true
  } catch (error) {
    open.value = false
    console.error('handleError', error, typeof error)
    add({ color: 'error', description: `${error}` })
  }
} */

const { syncLocaleAsync, syncThemeAsync, syncVaultNameAsync } =
  useVaultSettingsStore()

const check = ref(false)

const initVault = () => {
  vault.name = ''
  vault.password = ''
  vault.path = ''
  vault.type = 'password'
}

const onAbort = () => {
  initVault()
  open.value = false
}

const { add } = useToast()

const onOpenDatabase = async () => {
  try {
    if (!props.path) return

    const { openAsync } = useVaultStore()
    const localePath = useLocalePath()

    check.value = true
    const path = props.path
    const pathCheck = vaultSchema.path.safeParse(path)
    const passwordCheck = vaultSchema.password.safeParse(vault.password)

    if (pathCheck.error || passwordCheck.error) return

    const vaultId = await openAsync({
      path,
      password: vault.password,
    })

    if (!vaultId) {
      add({
        color: 'error',
        description: t('error.open'),
      })
      return
    }

    onAbort()

    await navigateTo(
      localePath({
        name: 'vaultOverview',
        params: {
          vaultId,
        },
      }),
    )
    await Promise.allSettled([
      syncLocaleAsync(),
      syncThemeAsync(),
      syncVaultNameAsync(),
    ])
  } catch (error) {
    open.value = false
    console.error('handleError', error, typeof error)
    add({ color: 'error', description: `${error}` })
  }
}
</script>

<i18n lang="yaml">
de:
  open: Entsperren
  title: '{haexvault} entsperren'
  password: Passwort
  vault:
    open: Vault öffnen
  description: Öffne eine vorhandene Vault
  error:
    open: Vault konnte nicht geöffnet werden. \n Vermutlich ist das Passwort falsch

en:
  open: Unlock
  title: Unlock {haexvault}
  password: Passwort
  description: Open your existing vault
  vault:
    open: Open Vault
  error:
    open: Vault couldn't be opened. \n The password is probably wrong
</i18n>

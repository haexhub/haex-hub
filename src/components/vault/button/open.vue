<template>
  <UiDialogConfirm
    v-model:open="open"
    class="btn btn-primary btn-outline shadow-md btn-lg"
    :confirm-label="t('open')"
    :abort-label="t('abort')"
    @abort="onAbort"
    @confirm="onOpenDatabase"
    @open="onLoadDatabase"
  >
    <template #title>
      <i18n-t
        keypath="title"
        tag="p"
        class="flex gap-x-2 flex-wrap"
      >
        <template #haexvault>
          <UiTextGradient>HaexVault</UiTextGradient>
        </template>
      </i18n-t>

      <div class="text-sm">{{ props.path ?? database.path }}</div>
    </template>

    <template #trigger>
      <Icon name="mdi:folder-open-outline" />
      {{ t('database.open') }}
    </template>

    <UiInputPassword
      :check-input="check"
      :rules="vaultDatabaseSchema.password"
      @keyup.enter="onOpenDatabase"
      autofocus
      prepend-icon="mdi:key-outline"
      v-model="database.password"
    />
  </UiDialogConfirm>
</template>

<script setup lang="ts">
import { open as openVault } from '@tauri-apps/plugin-dialog'
import { vaultDatabaseSchema } from './schema'

const { t } = useI18n()

const open = defineModel('open', { type: Boolean })

const props = defineProps<{
  path: string
}>()

const check = ref(false)

const database = reactive<{
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

const initDatabase = () => {
  database.name = ''
  database.password = ''
  database.path = ''
  database.type = 'password'
}

initDatabase()

const { add } = useSnackbar()

const handleError = (error: unknown) => {
  open.value = false
  console.error('handleError', error, typeof error)
  add({ type: 'error', text: `${error}` })
}

const { openAsync } = useVaultStore()

const onLoadDatabase = async () => {
  try {
    database.path = await openVault({
      multiple: false,
      directory: false,
      filters: [
        {
          name: 'HaexVault',
          extensions: ['db'],
        },
      ],
    })

    if (!database.path) return

    open.value = true
  } catch (error) {
    handleError(error)
  }
}

const localePath = useLocalePath()

const { syncLocaleAsync, syncThemeAsync, syncVaultNameAsync } =
  useVaultSettingsStore()

const onOpenDatabase = async () => {
  try {
    check.value = true
    const path = database.path || props.path
    const pathCheck = vaultDatabaseSchema.path.safeParse(path)
    const passwordCheck = vaultDatabaseSchema.password.safeParse(
      database.password,
    )

    if (!pathCheck.success || !passwordCheck.success || !path) {
      add({
        type: 'error',
        text: `Params falsch. Path: ${pathCheck.error} | Password: ${passwordCheck.error}`,
      })
      return
    }

    const vaultId = await openAsync({
      path,
      password: database.password,
    })

    if (!vaultId) {
      add({
        type: 'error',
        text: 'Vault konnte nicht geöffnet werden. \n Vermutlich ist das Passwort falsch',
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
    handleError(error)
  }
}

const onAbort = () => {
  initDatabase()
  open.value = false
}
</script>

<i18n lang="yaml">
de:
  open: Öffnen
  abort: Abbrechen
  title: '{haexvault} entsperren'
  database:
    open: Vault öffnen

en:
  open: Open
  abort: Abort
  title: Unlock {haexvault}
  database:
    open: Open Vault
</i18n>

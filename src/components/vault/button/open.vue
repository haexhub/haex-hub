<template>
  <UiDialog v-model:open="isOpen" class="btn btn-primary btn-outline shadow-md md:btn-lg shrink-0 flex-1 "
    @open="onLoadDatabase">

    <template #trigger>

      <Icon name="mdi:folder-open-outline" />
      {{ t('database.open') }}

    </template>

    <UiInputPassword :check-input="check" :rules="vaultDatabaseSchema.password" @keyup.enter="onOpenDatabase" autofocus
      prepend-icon="mdi:key-outline" v-model="database.password" />

    <template #buttons>
      <UiButton class="btn-error" @click="onClose">
        {{ t('abort') }}
      </UiButton>

      <UiButton type="submit" class="btn-primary" @click="onOpenDatabase">
        {{ t('open') }}
      </UiButton>
    </template>
  </UiDialog>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { vaultDatabaseSchema } from './schema'

const { t } = useI18n()

const isOpen = defineModel('isOpen', { type: Boolean })

const props = defineProps({
  path: String,
})

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
  isOpen.value = false
  console.error('handleError', error, typeof error)
  add({ type: 'error', text: 'Passwort falsch' })
}

const { openAsync } = useVaultStore()

const onLoadDatabase = async () => {
  try {
    database.path = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: 'HaexVault',
          extensions: ['db'],
        },
      ],
    })

    console.log("database.path", database.path)
    if (!database.path) return

    isOpen.value = true
  } catch (error) {
    handleError(error)
  }
}

const localePath = useLocalePath()

const { syncLocaleAsync, syncThemeAsync, syncVaultNameAsync } = useVaultStore()
const onOpenDatabase = async () => {
  try {
    check.value = true
    const path = database.path || props.path
    const pathCheck = vaultDatabaseSchema.path.safeParse(path)
    const passwordCheck = vaultDatabaseSchema.password.safeParse(
      database.password
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

    onClose()

    await navigateTo(
      localePath({
        name: 'vaultOverview',
        params: {
          vaultId,
        },
      })
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

const onClose = () => {
  initDatabase()
  isOpen.value = false
}
</script>

<i18n lang="json">{
  "de": {
    "open": "Öffnen",
    "abort": "Abbrechen",
    "database": {
      "open": "Vault öffnen"
    }
  },
  "en": {
    "open": "Open",
    "abort": "Abort",
    "database": {
      "open": "Open Vault"
    }
  }
}</i18n>
<template>
  <UiDialog
    v-model:open="open"
    :title="t('title')"
    class="btn btn-primary btn-outline shadow-md md:btn-lg"
    @click="open = true"
  >
    <template #trigger>
      <Icon name="mdi:plus" />
      {{ t('database.create') }}
    </template>

    <form
      class="flex flex-col gap-4"
      @submit="onCreateAsync"
    >
      <UiInput
        v-model="database.name"
        :check-input="check"
        :label="t('database.label')"
        :placeholder="t('database.placeholder')"
        :rules="vaultDatabaseSchema.name"
        autofocus
        prepend-icon="mdi:safe"
      />

      <UiInputPassword
        v-model="database.password"
        :check-input="check"
        :rules="vaultDatabaseSchema.password"
        prepend-icon="mdi:key-outline"
      />
    </form>

    <template #buttons>
      <UiButton
        class="btn-error"
        @click="onClose"
      >
        {{ t('abort') }}
      </UiButton>

      <UiButton
        class="btn-primary"
        @click="onCreateAsync"
      >
        {{ t('create') }}
      </UiButton>
    </template>
  </UiDialog>
</template>

<script setup lang="ts">
import { save } from '@tauri-apps/plugin-dialog'
import { onKeyStroke } from '@vueuse/core'
import { useVaultStore } from '~/stores/vault'
import { vaultDatabaseSchema } from './schema'

onKeyStroke('Enter', (e) => {
  e.preventDefault()
  onCreateAsync()
})

const check = ref(false)
const open = ref()

const { t } = useI18n()

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
  database.name = t('database.name')
  database.password = ''
  database.path = ''
  database.type = 'password'
}

initDatabase()

const { add } = useSnackbar()
const { createAsync } = useVaultStore()

const onCreateAsync = async () => {
  check.value = true

  const nameCheck = vaultDatabaseSchema.name.safeParse(database.name)
  const passwordCheck = vaultDatabaseSchema.password.safeParse(
    database.password,
  )

  console.log(
    'checks',
    database.name,
    nameCheck,
    database.password,
    passwordCheck,
  )
  if (!nameCheck.success || !passwordCheck.success) return

  open.value = false
  try {
    database.path = await save({
      defaultPath: database.name.endsWith('.db')
        ? database.name
        : `${database.name}.db`,
    })

    console.log('data', database)

    if (database.path && database.password) {
      const vaultId = await createAsync({
        path: database.path,
        password: database.password,
      })

      console.log('vaultId', vaultId)
      if (vaultId) {
        await navigateTo(
          useLocaleRoute()({ name: 'vaultOverview', params: { vaultId } }),
        )
      }
    }
  } catch (error) {
    console.error(error)
    add({ type: 'error', text: JSON.stringify(error) })
  }
}

const onClose = () => {
  open.value = false
  initDatabase()
}
</script>

<i18n lang="json">
{
  "de": {
    "database": {
      "label": "Vaultname",
      "placeholder": "Vaultname",
      "create": "Neue Vault anlegen",
      "name": "HaexVault"
    },
    "title": "Neue Datenbank anlegen",
    "create": "Erstellen",
    "abort": "Abbrechen",
    "description": "Haex Vault für deine geheimsten Geheimnisse"
  },
  "en": {
    "database": {
      "label": "Vaultname",
      "placeholder": "Vaultname",
      "create": "Create new Vault",
      "name": "HaexVault"
    },
    "title": "Create New Database",
    "create": "Create",
    "abort": "Abort",
    "description": "Haex Vault for your most secret secrets"
  }
}
</i18n>

<template>
  <UiDialogConfirm
    v-model:open="open"
    class="btn btn-primary btn-outline shadow-md btn-lg"
    @click="open = true"
    @abort="open = false"
    @confirm="onCreateAsync"
    :confirm-label="t('create')"
  >
    <template #trigger>
      <Icon name="mdi:plus" />
      {{ t('database.create') }}
    </template>

    <template #title>
      <div class="flex gap-x-2 items-center">
        <Icon
          name="mdi:safe"
          class="text-primary"
        />
        <p>
          {{ t('title') }}
        </p>
      </div>
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

    <!--  <template #buttons>
      <UiButton
        class="btn-error btn-outline w-full sm:w-auto"
        @click="onClose"
      >
        <Icon name="mdi:x" />
        {{ t('abort') }}
      </UiButton>

      <UiButton
        class="btn-primary w-full sm:w-auto"
        @click="onCreateAsync"
      >
        {{ t('create') }}
      </UiButton>
    </template> -->
  </UiDialogConfirm>
</template>

<script setup lang="ts">
import { save } from '@tauri-apps/plugin-dialog'
import { onKeyStroke } from '@vueuse/core'
import { useVaultStore } from '~/stores/vault'
import { vaultDatabaseSchema } from './schema'
import { BaseDirectory, readFile, writeFile } from '@tauri-apps/plugin-fs'
import { resolveResource } from '@tauri-apps/api/path'
//import { convertFileSrc } from "@tauri-apps/api/tauri";

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
    const template_vault_path = await resolveResource('database/vault.db')

    const template_vault = await readFile(template_vault_path)

    database.path = await save({
      defaultPath: database.name.endsWith('.db')
        ? database.name
        : `${database.name}.db`,
    })

    if (!database.path) return

    await writeFile('temp_vault.db', template_vault, {
      baseDir: BaseDirectory.AppLocalData,
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
    add({ type: 'error', text: `${error}` })
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
    "title": "Neue Vault anlegen",
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
    "title": "Create New Vault",
    "create": "Create",
    "abort": "Abort",
    "description": "Haex Vault for your most secret secrets"
  }
}
</i18n>

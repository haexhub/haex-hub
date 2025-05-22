<template>
  <Dialog>
    <!-- class="btn btn-primary btn-outline shadow-md md:btn-lg shrink-0 flex-1 whitespace-nowrap flex-nowrap" -->
    <DialogTrigger as-child>
      <Button>

        <Icon name="mdi:plus" />
        {{ t('database.create') }}
      </Button>
    </DialogTrigger>

    <DialogContent>
      <DialogHeader>
        <DialogTitle>Edit profile</DialogTitle>
        <DialogDescription>
          Make changes to your profile here. Click save when you're done.
        </DialogDescription>
      </DialogHeader>

      <form class="flex flex-col gap-4" @submit="onCreateAsync">
        <Input :check-input="check" :label="t('database.label')" :placeholder="t('database.placeholder')"
          :rules="vaultDatabaseSchema.name" autofocus prepend-icon="mdi:safe" v-model="database.name" />

        <!-- <UiInputPassword :check-input="check" :rules="vaultDatabaseSchema.password" prepend-icon="mdi:key-outline"
          v-model="database.password" /> -->
      </form>

      <DialogFooter>
        <Button class="btn-error" @click="onClose">
          {{ t('abort') }}
        </Button>

        <Button class="btn-primary" @click="onCreateAsync">
          {{ t('create') }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import { save } from '@tauri-apps/plugin-dialog'
import { onKeyStroke } from '@vueuse/core'
import { useVaultStore } from '~/stores/vault'
import { vaultDatabaseSchema } from './schema'
import { toast } from 'vue-sonner'

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


const { createAsync } = useVaultStore()

const onCreateAsync = async () => {
  check.value = true

  const nameCheck = vaultDatabaseSchema.name.safeParse(database.name)
  const passwordCheck = vaultDatabaseSchema.password.safeParse(
    database.password
  )

  console.log(
    'checks',
    database.name,
    nameCheck,
    database.password,
    passwordCheck
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
          useLocaleRoute()({ name: 'vaultOverview', params: { vaultId } })
        )
      }
    }
  } catch (error) {
    console.error(error)
    toast({ type: 'error', text: JSON.stringify(error) })
  }
}

const onClose = () => {
  open.value = false
  initDatabase()
}
</script>

<i18n lang="json">{
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
}</i18n>

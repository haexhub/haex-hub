<template>
  <UiDrawer
    v-model:open="open"
    :title="t('title')"
    :description="t('description')"
  >
    <UiButton
      :label="t('button.label')"
      :ui="{
        base: 'px-3 py-2',
      }"
      icon="mdi:plus"
      size="xl"
      variant="outline"
      block
    />

    <template #content>
      <div class="p-6 flex flex-col min-h-[50vh]">
        <div class="flex-1 flex items-center justify-center px-4">
          <UForm
            :state="vault"
            class="w-full max-w-md space-y-6"
          >
            <UFormField
              :label="t('vault.label')"
              name="name"
            >
              <UInput
                v-model="vault.name"
                icon="mdi:safe"
                :placeholder="t('vault.placeholder')"
                autofocus
                size="xl"
                class="w-full"
              />
            </UFormField>

            <UFormField
              :label="t('password.label')"
              name="password"
            >
              <UiInput
                v-model="vault.password"
                type="password"
                icon="i-heroicons-key"
                :placeholder="t('password.placeholder')"
                size="xl"
                class="w-full"
              />
            </UFormField>
          </UForm>
        </div>

        <div class="flex gap-3 mt-auto pt-6">
          <UButton
            color="neutral"
            variant="outline"
            block
            size="xl"
            @click="open = false"
          >
            {{ t('cancel') }}
          </UButton>
          <UButton
            color="primary"
            block
            size="xl"
            @click="onCreateAsync"
          >
            {{ t('create') }}
          </UButton>
        </div>
      </div>
    </template>
  </UiDrawer>
</template>

<script setup lang="ts">
import { vaultSchema } from './schema'

const open = defineModel<boolean>('open', { default: false })

const { t } = useI18n({
  useScope: 'local',
})

const vault = reactive<{
  name: string
  password: string
  type: 'password' | 'text'
}>({
  name: 'HaexVault',
  password: '',
  type: 'password',
})

const initVault = () => {
  vault.name = 'HaexVault'
  vault.password = ''
  vault.type = 'password'
}

const { createAsync } = useVaultStore()
const { add } = useToast()

const check = ref(false)

const onCreateAsync = async () => {
  check.value = true

  const nameCheck = vaultSchema.name.safeParse(vault.name)
  const passwordCheck = vaultSchema.password.safeParse(vault.password)

  if (!nameCheck.success || !passwordCheck.success) return

  open.value = false
  try {
    if (vault.name && vault.password) {
      const vaultId = await createAsync({
        vaultName: vault.name,
        password: vault.password,
      })

      if (vaultId) {
        initVault()
        await navigateTo(
          useLocaleRoute()({ name: 'desktop', params: { vaultId } }),
        )
      }
    }
  } catch (error) {
    console.error(error)
    add({ color: 'error', description: JSON.stringify(error) })
  }
}
</script>

<i18n lang="yaml">
de:
  button:
    label: Vault erstellen
  vault:
    label: Vaultname
    placeholder: Vaultname
  password:
    label: Passwort
    placeholder: Passwort eingeben
  title: Neue HaexVault erstellen
  create: Erstellen
  cancel: Abbrechen
  description: Erstelle eine neue Vault für deine Daten

en:
  button:
    label: Create vault
  vault:
    label: Vault name
    placeholder: Vault name
  password:
    label: Password
    placeholder: Enter password
  title: Create new HaexVault
  create: Create
  cancel: Cancel
  description: Create a new vault for your data
</i18n>

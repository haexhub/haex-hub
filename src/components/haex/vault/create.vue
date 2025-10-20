<template>
  <UiDialogConfirm
    :confirm-label="t('create')"
    @confirm="onCreateAsync"
  >
    <UiButton
      :label="t('vault.create')"
      :ui="{
        base: 'px-3 py-2',
      }"
      icon="mdi:plus"
      size="xl"
      variant="outline"
      block
    />

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
    </template>

    <template #body>
      <UForm
        :state="vault"
        class="flex flex-col gap-4 w-full h-full justify-center"
      >
        <UiInput
          v-model="vault.name"
          leading-icon="mdi:safe"
          :label="t('vault.label')"
          :placeholder="t('vault.placeholder')"
        />
        <UiInputPassword
          v-model="vault.password"
          leading-icon="mdi:key-outline"
        />

        <UButton
          hidden
          type="submit"
          @click="onCreateAsync"
        />
      </UForm>
    </template>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
import { vaultSchema } from './schema'

const { t } = useI18n()

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
const open = ref()

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
  vault:
    create: Neue Vault erstellen
    label: Vaultname
    placeholder: Vaultname
    name: HaexVault
  title: Neue {haexvault} erstellen
  create: Erstellen

en:
  vault:
    create: Create new vault
    label: Vaultname
    placeholder: Vaultname
    name: HaexVault
  title: Create new {haexvault}
  create: Create
</i18n>

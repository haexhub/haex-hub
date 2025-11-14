<template>
  <UiDrawer
    v-model:open="open"
    :title="t('title')"
    :description="path || t('description')"
  >
    <UiButton
      :label="t('button.label')"
      :ui="{
        base: 'px-3 py-2',
      }"
      icon="mdi:folder-open-outline"
      size="xl"
      variant="outline"
      block
    />

    <template #content>
      <div class="p-6 flex flex-col min-h-[50vh]">
        <div class="flex-1 flex items-center justify-center px-4">
          <div class="w-full max-w-md space-y-4">
            <div
              v-if="path"
              class="text-sm text-gray-500 dark:text-gray-400"
            >
              <span class="font-medium">{{ t('path.label') }}:</span>
              {{ path }}
            </div>

            <UForm
              :state="vault"
              class="w-full"
            >
              <UFormField
                :label="t('password.label')"
                name="password"
              >
                <UInput
                  v-model="vault.password"
                  type="password"
                  icon="i-heroicons-key"
                  :placeholder="t('password.placeholder')"
                  autofocus
                  size="xl"
                  class="w-full"
                  @keyup.enter="onOpenDatabase"
                />
              </UFormField>
            </UForm>
          </div>
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
            @click="onOpenDatabase"
          >
            {{ t('open') }}
          </UButton>
        </div>
      </div>
    </template>
  </UiDrawer>
</template>

<script setup lang="ts">
/* import { open as openVault } from '@tauri-apps/plugin-dialog' */
import { vaultSchema } from './schema'

const open = defineModel<boolean>('open', { default: false })
const props = defineProps<{
  path?: string
}>()

const { t } = useI18n({
  useScope: 'local',
})

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
        name: 'desktop',
        params: {
          vaultId,
        },
      }),
    )
  } catch (error) {
    open.value = false
    const errorDetails =
      error && typeof error === 'object' && 'details' in error
        ? (error as { details?: { reason?: string } }).details
        : undefined

    if (errorDetails?.reason === 'file is not a database') {
      add({
        color: 'error',
        title: t('error.password.title'),
        description: t('error.password.description'),
      })
    } else {
      add({ color: 'error', description: JSON.stringify(error) })
    }
  }
}
</script>

<i18n lang="yaml">
de:
  button:
    label: Vault öffnen
  open: Entsperren
  cancel: Abbrechen
  title: HaexVault entsperren
  path:
    label: Pfad
  password:
    label: Passwort
    placeholder: Passwort eingeben
  description: Öffne eine vorhandene Vault
  error:
    open: Vault konnte nicht geöffnet werden
    password:
      title: Vault konnte nicht geöffnet werden
      description: Bitte überprüfe das Passwort

en:
  button:
    label: Open Vault
  open: Unlock
  cancel: Cancel
  title: Unlock HaexVault
  path:
    label: Path
  password:
    label: Password
    placeholder: Enter password
  description: Open your existing vault
  error:
    open: Vault couldn't be opened
    password:
      title: Vault couldn't be opened
      description: Please check your password
</i18n>

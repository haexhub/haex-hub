<template>
  <UiDialogConfirm
    v-model:open="open"
    @abort="onDeny"
    @confirm="onConfirm"
  >
    <template #title>
      {{ t('title') }}
    </template>

    <template #body>
      <div class="flex flex-col gap-6">
        <!-- Extension Info -->
        <UCard>
          <div class="flex items-start gap-4">
            <div
              v-if="preview?.manifest.icon"
              class="w-16 h-16 flex-shrink-0"
            >
              <UIcon
                :name="preview.manifest.icon"
                class="w-full h-full"
              />
            </div>
            <div class="flex-1">
              <h3 class="text-xl font-bold">
                {{ preview?.manifest.name }}
              </h3>
              <p class="text-sm text-gray-500 dark:text-gray-400">
                {{ t('version') }}: {{ preview?.manifest.version }}
              </p>
              <p
                v-if="preview?.manifest.author"
                class="text-sm text-gray-500 dark:text-gray-400"
              >
                {{ t('author') }}: {{ preview.manifest.author }}
              </p>
              <p
                v-if="preview?.manifest.description"
                class="text-sm mt-2"
              >
                {{ preview.manifest.description }}
              </p>

              <!-- Signature Verification -->
              <UBadge
                :color="preview?.is_valid_signature ? 'success' : 'error'"
                variant="subtle"
                class="mt-2"
              >
                <template #leading>
                  <UIcon
                    :name="
                      preview?.is_valid_signature
                        ? 'i-heroicons-shield-check'
                        : 'i-heroicons-shield-exclamation'
                    "
                  />
                </template>
                {{
                  preview?.is_valid_signature
                    ? t('signature.valid')
                    : t('signature.invalid')
                }}
              </UBadge>
            </div>
          </div>
        </UCard>

        <!-- Add to Desktop Option -->
        <UCheckbox
          v-model="addToDesktop"
          :label="t('addToDesktop')"
        />

        <!-- Permissions Section -->
        <div class="flex flex-col gap-4">
          <h4 class="text-lg font-semibold">
            {{ t('permissions.title') }}
          </h4>

          <UAccordion
            :items="permissionAccordionItems"
            :ui="{ root: 'flex flex-col gap-2' }"
          >
            <template #database>
              <div
                v-if="databasePermissions"
                class="pb-4"
              >
                <HaexExtensionPermissionList
                  v-model="databasePermissions"
                  :title="t('permissions.database')"
                />
              </div>
            </template>

            <template #filesystem>
              <div
                v-if="filesystemPermissions"
                class="pb-4"
              >
                <HaexExtensionPermissionList
                  v-model="filesystemPermissions"
                  :title="t('permissions.filesystem')"
                />
              </div>
            </template>

            <template #http>
              <div
                v-if="httpPermissions"
                class="pb-4"
              >
                <HaexExtensionPermissionList
                  v-model="httpPermissions"
                  :title="t('permissions.http')"
                />
              </div>
            </template>

            <template #shell>
              <div
                v-if="shellPermissions"
                class="pb-4"
              >
                <HaexExtensionPermissionList
                  v-model="shellPermissions"
                  :title="t('permissions.shell')"
                />
              </div>
            </template>
          </UAccordion>
        </div>
      </div>
    </template>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
import type { ExtensionPreview } from '~~/src-tauri/bindings/ExtensionPreview'

const { t } = useI18n()

const open = defineModel<boolean>('open', { default: false })
const preview = defineModel<ExtensionPreview | null>('preview', {
  default: null,
})
const addToDesktop = ref(true)

const databasePermissions = computed({
  get: () => preview.value?.editable_permissions?.database || [],
  set: (value) => {
    if (preview.value?.editable_permissions) {
      preview.value.editable_permissions.database = value
    }
  },
})

const filesystemPermissions = computed({
  get: () => preview.value?.editable_permissions?.filesystem || [],
  set: (value) => {
    if (preview.value?.editable_permissions) {
      preview.value.editable_permissions.filesystem = value
    }
  },
})

const httpPermissions = computed({
  get: () => preview.value?.editable_permissions?.http || [],
  set: (value) => {
    if (preview.value?.editable_permissions) {
      preview.value.editable_permissions.http = value
    }
  },
})

const shellPermissions = computed({
  get: () => preview.value?.editable_permissions?.shell || [],
  set: (value) => {
    if (preview.value?.editable_permissions) {
      preview.value.editable_permissions.shell = value
    }
  },
})


const permissionAccordionItems = computed(() => {
  const items = []

  if (databasePermissions.value?.length) {
    items.push({
      label: t('permissions.database'),
      icon: 'i-heroicons-circle-stack',
      slot: 'database',
      defaultOpen: true,
    })
  }

  if (filesystemPermissions.value?.length) {
    items.push({
      label: t('permissions.filesystem'),
      icon: 'i-heroicons-folder',
      slot: 'filesystem',
    })
  }

  if (httpPermissions.value?.length) {
    items.push({
      label: t('permissions.http'),
      icon: 'i-heroicons-globe-alt',
      slot: 'http',
    })
  }

  if (shellPermissions.value?.length) {
    items.push({
      label: t('permissions.shell'),
      icon: 'i-heroicons-command-line',
      slot: 'shell',
    })
  }

  return items
})

const emit = defineEmits<{
  deny: []
  confirm: [addToDesktop: boolean]
}>()

const onDeny = () => {
  open.value = false
  emit('deny')
}

const onConfirm = () => {
  open.value = false
  emit('confirm', addToDesktop.value)
}
</script>

<i18n lang="yaml">
de:
  title: Erweiterung installieren
  version: Version
  author: Autor
  addToDesktop: Zum Desktop hinzufügen
  signature:
    valid: Signatur verifiziert
    invalid: Signatur ungültig
  permissions:
    title: Berechtigungen
    database: Datenbank
    filesystem: Dateisystem
    http: Internet
    shell: Terminal

en:
  title: Install Extension
  version: Version
  author: Author
  addToDesktop: Add to Desktop
  signature:
    valid: Signature verified
    invalid: Invalid signature
  permissions:
    title: Permissions
    database: Database
    filesystem: Filesystem
    http: Internet
    shell: Terminal
</i18n>

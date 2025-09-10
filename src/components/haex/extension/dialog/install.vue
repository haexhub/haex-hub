<template>
  <UiDialogConfirm
    v-model:open="open"
    @abort="onDeny"
    @confirm="onConfirm"
  >
    <template #title>
      <i18n-t
        keypath="question"
        tag="p"
      >
        <template #extension>
          <span class="font-bold text-primary">{{ manifest?.name }}</span>
        </template>
      </i18n-t>
    </template>

    <div class="flex flex-col">
      <nav
        class="tabs tabs-bordered"
        aria-label="Tabs"
        role="tablist"
        aria-orientation="horizontal"
      >
        <button
          v-show="manifest?.permissions?.database"
          id="tabs-basic-item-1"
          type="button"
          class="tab active-tab:tab-active active"
          data-tab="#tabs-basic-1"
          aria-controls="tabs-basic-1"
          role="tab"
          aria-selected="true"
        >
          {{ t('database') }}
        </button>
        <button
          v-show="manifest?.permissions?.filesystem"
          id="tabs-basic-item-2"
          type="button"
          class="tab active-tab:tab-active"
          data-tab="#tabs-basic-2"
          aria-controls="tabs-basic-2"
          role="tab"
          aria-selected="false"
        >
          {{ t('filesystem') }}
        </button>
        <button
          v-show="manifest?.permissions?.http"
          id="tabs-basic-item-3"
          type="button"
          class="tab active-tab:tab-active"
          data-tab="#tabs-basic-3"
          aria-controls="tabs-basic-3"
          role="tab"
          aria-selected="false"
        >
          {{ t('http') }}
        </button>
      </nav>

      <div class="mt-3 min-h-40">
        <div
          id="tabs-basic-1"
          role="tabpanel"
          aria-labelledby="tabs-basic-item-1"
        >
          <HaexExtensionManifestPermissionsDatabase
            :database="permissions?.database"
          />
        </div>
        <div
          id="tabs-basic-2"
          class="hidden"
          role="tabpanel"
          aria-labelledby="tabs-basic-item-2"
        >
          <HaexExtensionManifestPermissionsFilesystem
            :filesystem="permissions?.filesystem"
          />
        </div>
        <div
          id="tabs-basic-3"
          class="hidden"
          role="tabpanel"
          aria-labelledby="tabs-basic-item-3"
        >
          <HaexExtensionManifestPermissionsHttp :http="permissions?.http" />
        </div>
      </div>
    </div>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
import type { IHaexHubExtensionManifest } from '~/types/haexhub'

const { t } = useI18n()

const open = defineModel<boolean>('open', { default: false })
const { manifest } = defineProps<{
  manifest?: IHaexHubExtensionManifest | null
}>()

const permissions = computed(() => ({
  database: {
    read: manifest?.permissions.database?.read?.map((read) => ({
      [read]: true,
    })),
    write: manifest?.permissions.database?.read?.map((write) => ({
      [write]: true,
    })),
    create: manifest?.permissions.database?.read?.map((create) => ({
      [create]: true,
    })),
  },

  filesystem: {
    read: manifest?.permissions.filesystem?.read?.map((read) => ({
      [read]: true,
    })),
    write: manifest?.permissions.filesystem?.write?.map((write) => ({
      [write]: true,
    })),
  },

  http: manifest?.permissions.http?.map((http) => ({
    [http]: true,
  })),
}))

watch(permissions, () => console.log('permissions', permissions.value))
const emit = defineEmits(['deny', 'confirm'])

const onDeny = () => {
  open.value = false
  console.log('onDeny open', open.value)
  emit('deny')
}

const onConfirm = () => {
  open.value = false
  console.log('onConfirm open', open.value)
  emit('confirm')
}
</script>

<i18n lang="json">
{
  "de": {
    "title": "Erweiterung hinzufügen",
    "question": "Erweiterung {extension} hinzufügen?",
    "confirm": "Bestätigen",
    "deny": "Ablehnen",
    "database": "Datenbank",
    "http": "Internet",
    "filesystem": "Dateisystem"
  },
  "en": {
    "title": "Confirm Permission",
    "question": "Add Extension {extension}?",
    "confirm": "Confirm",
    "deny": "Deny",
    "database": "Database",
    "http": "Internet",
    "filesystem": "Filesystem"
  }
}
</i18n>

<template>
  <div
    v-if="menuEntry"
    class="flex items-center justify-between gap-4 p-3 rounded-lg border border-base-300 bg-base-100"
  >
    <div class="flex-1 min-w-0">
      <div class="font-medium truncate">
        {{ modelValue.target }}
      </div>
      <div
        v-if="modelValue.operation"
        class="text-sm text-gray-500 dark:text-gray-400"
      >
        {{ t(`operation.${modelValue.operation}`) }}
      </div>
    </div>

    <div class="flex items-center gap-2">
      <!-- Status Selector -->
      <USelectMenu
        v-model="menuEntry"
        :items="statusOptions"
        value-attribute="value"
        class="w-44"
      >
        <template #leading>
          <UIcon
            :name="getStatusIcon(menuEntry?.value)"
            :class="getStatusColor(menuEntry?.value)"
          />
        </template>

        <template #item-leading="{ item }">
          <UIcon
            :name="getStatusIcon(item?.value)"
            :class="getStatusColor(item?.value)"
          />
        </template>
      </USelectMenu>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PermissionEntry } from '~~/src-tauri/bindings/PermissionEntry'
import type { PermissionStatus } from '~~/src-tauri/bindings/PermissionStatus'

const permissionEntry = defineModel<PermissionEntry>({ required: true })

const menuEntry = computed({
  get: () =>
    statusOptions.value.find(
      (option) => option.value == permissionEntry.value.status,
    ),
  set(newStatus) {
    const status =
      statusOptions.value.find((option) => option.value == newStatus?.value)
        ?.value || 'denied'
    if (isPermissionStatus(status)) {
      permissionEntry.value.status = status
    } else {
      permissionEntry.value.status = 'denied'
    }
  },
})

const { t } = useI18n()

const isPermissionStatus = (value: string): value is PermissionStatus => {
  return ['ask', 'granted', 'denied'].includes(value)
}

const statusOptions = computed(() => [
  {
    value: 'granted',
    label: t('status.granted'),
    icon: 'i-heroicons-check-circle',
    color: 'text-green-500',
  },
  {
    value: 'ask',
    label: t('status.ask'),
    icon: 'i-heroicons-question-mark-circle',
    color: 'text-yellow-500',
  },
  {
    value: 'denied',
    label: t('status.denied'),
    icon: 'i-heroicons-x-circle',
    color: 'text-red-500',
  },
])

const getStatusIcon = (status: string) => {
  const option = statusOptions.value.find((o) => o.value === status)
  return option?.icon || 'i-heroicons-question-mark-circle'
}

const getStatusColor = (status: string) => {
  const option = statusOptions.value.find((o) => o.value === status)
  return option?.color || 'text-gray-500'
}
</script>

<i18n lang="yaml">
de:
  status:
    granted: Erlaubt
    ask: Nachfragen
    denied: Verweigert
  operation:
    read: Lesen
    write: Schreiben
    readWrite: Lesen & Schreiben
    request: Anfrage
    execute: Ausführen
en:
  status:
    granted: Granted
    ask: Ask
    denied: Denied
  operation:
    read: Read
    write: Write
    readWrite: Read & Write
    request: Request
    execute: Execute
</i18n>

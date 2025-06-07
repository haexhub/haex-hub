<template>
  <div class="">
    <div class="p-6">
      <UiButton
        class="btn-error"
        @click="onDeleteNotificationsAsync"
      >
        {{ t('delete') }}
      </UiButton>
    </div>
    <table class="table">
      <thead>
        <tr>
          <th>
            <input
              v-model="selectAll"
              type="checkbox"
              class="checkbox checkbox-primary checkbox-sm"
              aria-label="notification"
            />
          </th>
          <th>{{ t('title') }}</th>
          <th>{{ t('text') }}</th>
          <th>{{ t('date') }}</th>
          <th>{{ t('type') }}</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="notification in notifications"
          :key="notification.id"
        >
          <td>
            <label>
              <input
                v-model="selectedNotificationIds"
                :name="notification.id"
                :value="notification.id"
                aria-label="notification"
                class="checkbox checkbox-primary checkbox-sm"
                type="checkbox"
              />
            </label>
          </td>
          <td>{{ notification.title }}</td>
          <td>{{ notification.text }}</td>
          <td>
            {{
              notification.date
                ? new Date(notification.date).toLocaleDateString(locale, {
                    dateStyle: 'short',
                  })
                : ''
            }}
          </td>
          <td>
            <span
              class="badge badge-soft text-xs"
              :class="badgeClass[notification.type]"
            >
              {{ t(`types.${notification.type}`) }}
            </span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexNotifications } from '~~/src-tauri/database/schemas/vault'

definePageMeta({
  name: 'notifications',
})

const { t, locale } = useI18n()

const notifications = ref<SelectHaexNotifications[]>([])

const { deleteNotificationsAsync, syncNotificationsAsync } =
  useNotificationStore()

onMounted(async () => {
  await syncNotificationsAsync()
})

const selectedNotificationIds = ref<string[]>([])
const selectAll = computed({
  get() {
    return (
      notifications.value.length > 0 &&
      notifications.value.length === selectedNotificationIds.value.length
    )
  },
  set(value: boolean) {
    selectedNotificationIds.value = value
      ? [...notifications.value.map((notification) => notification.id)]
      : []
  },
})

const { add } = useSnackbar()

const onDeleteNotificationsAsync = async () => {
  try {
    console.log('onDeleteNotificationsAsync', selectedNotificationIds.value)
    await deleteNotificationsAsync(selectedNotificationIds.value)
  } catch (error) {
    console.error(error)
    add({ type: 'error', text: JSON.stringify(error) })
  }
}

const badgeClass: Record<SelectHaexNotifications['type'], string> = {
  error: 'badge-error',
  info: 'badge-info',
  success: 'badge-success',
  warning: 'badge-warning',
  log: 'badge-accent',
}
</script>

<i18n lang="yaml">
de:
  title: Titel
  text: Text
  type: Typ
  date: Datum
  delete: Benachrichtigungen löschen
  types:
    error: Fehler
    info: Info
    success: Erfolg
    warning: Warnung

en:
  title: Title
  text: Text
  type: Type
  date: Date
  delete: Delete Notifications
  types:
    error: Error
    info: Info
    success: Success
    warning: Warning
</i18n>

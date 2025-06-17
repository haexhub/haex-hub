<template>
  <div
    class="dropdown relative inline-flex [--auto-close:inside] [--offset:18] [--placement:bottom]"
  >
    <UiTooltip :tooltip="t('notifications.label')">
      <button
        id="dropdown-scrollable"
        type="button"
        class="dropdown-toggle btn btn-text btn-circle dropdown-open:bg-base-content/10"
        aria-haspopup="menu"
        aria-expanded="false"
        aria-label="Dropdown"
      >
        <div class="indicator">
          <span
            v-show="notifications.length"
            class="indicator-item bg-error size-2 rounded-full text-sm"
          />
          <span class="icon-[tabler--bell] text-base-content size-[1.375rem]" />
        </div>
      </button>
    </UiTooltip>
    <div
      class="dropdown-menu dropdown-open:opacity-100 hidden w-full max-w-96 shadow"
      role="menu"
      aria-orientation="vertical"
      aria-labelledby="dropdown-scrollable"
    >
      <div class="dropdown-header justify-center">
        <h6 class="text-base-content text-base">
          {{ t('notifications.label') }}
        </h6>
      </div>
      <div
        class="vertical-scrollbar horizontal-scrollbar rounded-scrollbar text-base-content/80 max-h-56 overflow-auto"
      >
        <div
          v-for="notification in notifications"
          :key="notification.id"
          class="dropdown-item"
        >
          <div class="avatar">
            <div class="w-10 rounded-full">
              <img
                v-if="notification.image"
                :src="notification.image"
                :alt="notification.alt ?? 'notification avatar'"
              />
              <Icon
                v-else-if="notification.icon"
                :name="notification.icon"
              />
            </div>
          </div>
          <div class="w-60">
            <h6 class="truncate text-base">
              {{ notification.title }}
            </h6>
            <small class="text-base-content/50 truncate">
              {{ notification.text }}
            </small>
          </div>
        </div>
      </div>
      <NuxtLinkLocale
        :to="{ name: 'notifications' }"
        class="dropdown-footer justify-center gap-1 hover:bg-base-content/10"
      >
        <span class="icon-[tabler--eye] size-4" />
        {{ t('notifications.view_all') }}
      </NuxtLinkLocale>
    </div>
  </div>
</template>

<script setup lang="ts">
const { t } = useI18n()
const { notifications } = storeToRefs(useNotificationStore())
</script>

<i18n lang="yaml">
de:
  notifications:
    label: Benachrichtigungen
    view_all: Alle ansehen
en:
  notifications:
    label: Notifications
    view_all: View all
</i18n>

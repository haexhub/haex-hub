<template>
  <div class="w-full h-full flex flex-col min-w-min relative overflow-hidden">
    <nav
      class="navbar bg-base-100 rounded-b max-sm:shadow border-b border-base-content/25 sm:z-20 relative px-2"
    >
      <UiTooltip :tooltip="isVisible ? t('sidebar.close') : t('sidebar.show')">
        <button
          type="button"
          class="btn btn-text btn-square me-2 z-50"
          aria-haspopup="dialog"
          aria-expanded="false"
          aria-controls="sidebar"
          @click="toogleSidebar"
          ref="sidebarToogleRef"
        >
          <Icon
            :name="
              isVisible
                ? 'tabler:layout-sidebar-filled'
                : 'tabler:layout-sidebar'
            "
            size="28"
          />
        </button>
      </UiTooltip>

      <div class="flex flex-1 items-center">
        <NuxtLinkLocale
          class="link text-base-content link-neutral text-xl font-semibold no-underline"
          :to="{ name: 'vaultOverview' }"
        >
          <UiTextGradient class="text-nowrap">{{
            currentVaultName
          }}</UiTextGradient>
        </NuxtLinkLocale>
      </div>

      <div class="navbar-end flex items-center gap-4 me-4">
        <div
          class="dropdown relative inline-flex [--auto-close:inside] [--offset:8] [--placement:bottom-end]"
        >
          <button
            id="dropdown-scrollable"
            type="button"
            class="dropdown-toggle btn btn-text btn-circle dropdown-open:bg-base-content/10 size-10"
            aria-haspopup="menu"
            aria-expanded="false"
            aria-label="Dropdown"
          >
            <div class="indicator">
              <span
                v-show="notifications.length"
                class="indicator-item bg-error size-2 rounded-full text-sm"
              ></span>
              <span
                class="icon-[tabler--bell] text-base-content size-[1.375rem]"
              ></span>
            </div>
          </button>
          <div
            class="dropdown-menu dropdown-open:opacity-100 hidden"
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
              class="vertical-scrollbar horizontal-scrollbar rounded-scrollbar text-base-content/80 max-h-56 overflow-auto max-md:max-w-60"
            >
              <div class="dropdown-item" v-for="notification in notifications">
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
                    {{ notification.description }}
                  </small>
                </div>
              </div>
            </div>
            <a href="#" class="dropdown-footer justify-center gap-1">
              <span class="icon-[tabler--eye] size-4"></span>
              {{ t('notifications.view_all') }}
            </a>
          </div>
        </div>

        <HaexMenuMain />
      </div>
    </nav>

    <div class="flex h-full overflow-hidden">
      <aside
        id="sidebar"
        class="sm:shadow-none transition-all h-full overflow-hidden border-r border-base-300"
        :class="[!isVisible ? 'w-0' : 'w-16']"
        role="dialog"
        tabindex="-1"
      >
        <div class="drawer-body h-full">
          <ul class="menu p-0 h-full rounded-none">
            <HaexSidebarLink
              v-bind="item"
              v-for="item in menu"
              :key="item.id"
            />
            <HaexSidebarLink
              v-for="item in extensionLinks"
              :key="item.id"
              v-bind="item"
              icon-type="svg"
            />
          </ul>
        </div>
      </aside>

      <main class="w-full h-full overflow-scroll">
        <NuxtPage :transition="{ name: 'fade' }" />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
const { t } = useI18n()

const { currentVaultName } = storeToRefs(useVaultStore())

const { menu, isVisible } = storeToRefs(useSidebarStore())

const { notifications } = storeToRefs(useNotificationStore())

const { extensionLinks } = storeToRefs(useExtensionsStore())

const toogleSidebar = () => {
  isVisible.value = !isVisible.value
}
</script>

<i18n lang="yaml">
de:
  notifications:
    label: Benachrichtigungen
    view_all: Alle ansehen
  vault:
    close: Vault schließen
  sidebar:
    close: Sidebar ausblenden
    show: Sidebar anzeigen
en:
  notifications:
    label: Notifications
    view_all: View all
  vault:
    close: Close vault
  sidebar:
    close: close sidebar
    show: show sidebar
</i18n>

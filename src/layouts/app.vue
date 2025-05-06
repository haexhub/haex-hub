<template>
  <div class="w-full h-full flex flex-col min-w-min">
    <nav
      class="navbar bg-base-100 rounded-b max-sm:shadow border-b border-base-content/25 sm:z-20 relative px-2"
    >
      <button
        type="button"
        class="btn btn-text btn-square me-2 z-50"
        aria-haspopup="dialog"
        aria-expanded="false"
        aria-controls="sidebar"
        @click="toogleSidebar"
        ref="sidebarToogleRef"
      >
        <Icon name="mage:dash-menu" size="28" />
      </button>

      <div class="flex flex-1 items-center">
        <NuxtLinkLocale
          class="link text-base-content link-neutral text-xl font-semibold no-underline"
          :to="{ name: 'vaultOverview' }"
        >
          <UiTextGradient class="text-nowrap">Haex Hub</UiTextGradient>
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
              <span class="icon-[tabler--bell] text-base-content size-[1.375rem]"></span>
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
                {{ t("notifications.label") }}
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
                    <Icon v-else-if="notification.icon" :name="notification.icon" />
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
              {{ t("notifications.view_all") }}
            </a>
          </div>
        </div>
        <div
          class="dropdown relative inline-flex [--auto-close:inside] [--offset:8] [--placement:bottom-end]"
        >
          <button
            id="dropdown-scrollable"
            type="button"
            class="dropdown-toggle flex items-center"
            aria-haspopup="menu"
            aria-expanded="false"
            aria-label="Dropdown"
          >
            <div class="avatar">
              <div class="size-9.5 rounded-full">
                <img src="https://cdn.flyonui.com/fy-assets/avatar/avatar-1.png" alt="avatar 1" />
              </div>
            </div>
          </button>
          <ul
            class="dropdown-menu dropdown-open:opacity-100 hidden min-w-60"
            role="menu"
            aria-orientation="vertical"
            aria-labelledby="dropdown-avatar"
          >
            <li class="dropdown-header gap-2">
              <div class="avatar">
                <div class="w-10 rounded-full">
                  <img src="https://cdn.flyonui.com/fy-assets/avatar/avatar-1.png" alt="avatar" />
                </div>
              </div>
              <div>
                <h6 class="text-base-content text-base font-semibold">John Doe</h6>
                <small class="text-base-content/50">Admin</small>
              </div>
            </li>
            <li>
              <a class="dropdown-item" href="#">
                <span class="icon-[tabler--user]"></span>
                My Profile
              </a>
            </li>
            <li>
              <a class="dropdown-item" href="#">
                <span class="icon-[tabler--settings]"></span>
                Settings
              </a>
            </li>
            <li>
              <a class="dropdown-item" href="#">
                <span class="icon-[tabler--receipt-rupee]"></span>
                Billing
              </a>
            </li>
            <li>
              <a class="dropdown-item" href="#">
                <span class="icon-[tabler--help-triangle]"></span>
                FAQs
              </a>
            </li>
            <li class="dropdown-footer gap-2">
              <button class="btn btn-error btn-soft btn-block" @click="onVaultCloseAsync">
                <span class="icon-[tabler--logout]"></span>
                {{ t("vault.close") }}
              </button>
            </li>
          </ul>
        </div>
      </div>
    </nav>

    <div class="flex h-full">
      <aside
        id="sidebar"
        class="sm:shadow-none drawer max-w-14 transition-all"
        :class="[!isVisible ? 'w-0' : 'w-14']"
        role="dialog"
        tabindex="-1"
      >
        <div class="drawer-body px-0">
          <ul class="menu p-0">
            <UiSidebarLink v-bind="item" v-for="item in menu" :key="item.id" />
            <UiSidebarLinkExtension
              v-bind="item"
              v-for="item in availableExtensions"
              :key="item.id"
            />
          </ul>
        </div>
      </aside>

      <div class="overflow-hidden transition-all relative w-full">
        <div
          class="h-full overflow-scroll transition-all pl-0"
          :class="[isVisible ? 'sm:pl-14 ' : ' pl-0']"
        >
          <slot />
        </div>
      </div>
    </div>
    <!--  <main class="sm:pl-14">
      <NuxtPage />
    </main> -->
  </div>
</template>

<script setup lang="ts">
import { NuxtLinkLocale } from "#components";

const { t } = useI18n();
const { menu, isVisible } = storeToRefs(useSidebarStore());
const sidebarToogleRef = useTemplateRef("sidebarToogleRef");
onClickOutside(sidebarToogleRef, () => {
  if (currentScreenSize.value === "xs") {
    isVisible.value = false;
  }
});
const { notifications } = storeToRefs(useNotificationStore());

const { isActive } = useExtensionsStore();
const { closeAsync } = useVaultStore();
const { currentScreenSize } = storeToRefs(useUiStore());
const onExtensionSelectAsync = async (id: string) => {};
const { availableExtensions } = storeToRefs(useExtensionsStore());
const toogleSidebar = () => {
  isVisible.value = !isVisible.value;
};

const onVaultCloseAsync = async () => {
  await closeAsync();
  await navigateTo(useLocalePath()({ name: "vaultOpen" }));
};
</script>

<i18n lang="yaml">
de:
  notifications:
    label: Benachrichtigungen
    view_all: Alle ansehen
  vault:
    close: Vault schließen
en:
  notifications:
    label: Notifications
    view_all: View all
  vault:
    close: Close vault
</i18n>

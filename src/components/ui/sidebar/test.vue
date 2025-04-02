<template>
  <nav
    class="navbar bg-base-100 max-sm:rounded-box max-sm:shadow sm:border-b border-base-content/25 sm:z-[1] relative"
  >
    <button
      type="button"
      class="btn btn-text max-sm:btn-square sm:hidden me-2"
      aria-haspopup="dialog"
      aria-expanded="false"
      aria-controls="sidebar"
      data-overlay="#sidebar"
    >
      <span class="icon-[tabler--menu-2] size-5"></span>
    </button>
    <div class="flex flex-1 items-center">
      <a
        class="link text-base-content link-neutral text-xl font-semibold no-underline"
        href="#"
      >
        <UiTextGradient>Haex Hub</UiTextGradient>
      </a>
    </div>
    <div class="navbar-end flex items-center gap-4">
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
            <div
              class="dropdown-item"
              v-for="notification in notifications"
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
                  {{ notification.description }}
                </small>
              </div>
            </div>
          </div>
          <a
            href="#"
            class="dropdown-footer justify-center gap-1"
          >
            <span class="icon-[tabler--eye] size-4"></span>
            {{ t('notifications.view_all') }}
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
              <img
                src="https://cdn.flyonui.com/fy-assets/avatar/avatar-1.png"
                alt="avatar 1"
              />
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
                <img
                  src="https://cdn.flyonui.com/fy-assets/avatar/avatar-1.png"
                  alt="avatar"
                />
              </div>
            </div>
            <div>
              <h6 class="text-base-content text-base font-semibold">
                John Doe
              </h6>
              <small class="text-base-content/50">Admin</small>
            </div>
          </li>
          <li>
            <a
              class="dropdown-item"
              href="#"
            >
              <span class="icon-[tabler--user]"></span>
              My Profile
            </a>
          </li>
          <li>
            <a
              class="dropdown-item"
              href="#"
            >
              <span class="icon-[tabler--settings]"></span>
              Settings
            </a>
          </li>
          <li>
            <a
              class="dropdown-item"
              href="#"
            >
              <span class="icon-[tabler--receipt-rupee]"></span>
              Billing
            </a>
          </li>
          <li>
            <a
              class="dropdown-item"
              href="#"
            >
              <span class="icon-[tabler--help-triangle]"></span>
              FAQs
            </a>
          </li>
          <li class="dropdown-footer gap-2">
            <a
              class="btn btn-error btn-soft btn-block"
              href="#"
            >
              <span class="icon-[tabler--logout]"></span>
              Sign out
            </a>
          </li>
        </ul>
      </div>
    </div>
  </nav>

  <aside
    id="sidebar"
    class="overlay sm:shadow-none overlay-open:translate-x-0 drawer drawer-start hidden max-w-64 sm:absolute sm:z-0 sm:flex sm:translate-x-0 pt-16"
    role="dialog"
    tabindex="-1"
  >
    <div class="drawer-body px-2 pt-4">
      <ul class="menu p-0">



        <li v-for="item in menu">
          <a href="#">
            <span class="icon-[tabler--home] size-5"></span>
            Home
          </a>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--user] size-5"></span>
            Account
          </a>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--message] size-5"></span>
            Notifications
          </a>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--mail] size-5"></span>
            Email
          </a>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--calendar] size-5"></span>
            Calendar
          </a>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--shopping-bag] size-5"></span>
            Product
          </a>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--login] size-5"></span>
            Sign In
          </a>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--logout-2] size-5"></span>
            Sign Out
          </a>
        </li>
      </ul>
    </div>
  </aside>
</template>

<script setup lang="ts">
const { t } = useI18n();

const { notifications } = storeToRefs(useNotificationStore());

const { menu } = storeToRefs(useSidebarStore());
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

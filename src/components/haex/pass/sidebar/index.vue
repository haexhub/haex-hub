<template>
  <aside
    :id
    ref="sidebarRef"
    class="flex sm:shadow-none w-full md:max-w-64"
    tabindex="-1"
  >
    <div class="drawer-body w-full">
      <ul class="menu space-y-0.5 p-0 rounded-none md:rounded">
        <li>
          <a href="#">
            <span class="icon-[tabler--home] size-5" />
            Home
          </a>
        </li>
        <li class="space-y-0.5">
          <a
            id="menu-app"
            class="collapse-toggle collapse-open:bg-base-content/10"
            data-collapse="#menu-app-collapse"
          >
            <span class="icon-[tabler--apps] size-5" />
            Apps
            <span
              class="icon-[tabler--chevron-down] collapse-open:rotate-180 size-4 transition-all duration-300"
            />
          </a>
          <ul
            id="menu-app-collapse"
            class="collapse hidden w-auto space-y-0.5 overflow-hidden transition-[height] duration-300"
            aria-labelledby="menu-app"
          >
            <li>
              <a href="#">
                <span class="icon-[tabler--message] size-5" />
                Chat
              </a>
            </li>
            <li>
              <a href="#">
                <span class="icon-[tabler--calendar] size-5" />
                Calendar
              </a>
            </li>
            <li class="space-y-0.5">
              <a
                id="sub-menu-academy"
                class="collapse-toggle collapse-open:bg-base-content/10"
                data-collapse="#sub-menu-academy-collapse"
              >
                <span class="icon-[tabler--book] size-5" />
                Academy
                <span
                  class="icon-[tabler--chevron-down] collapse-open:rotate-180 size-4"
                />
              </a>
              <ul
                id="sub-menu-academy-collapse"
                class="collapse hidden w-auto space-y-0.5 overflow-hidden transition-[height] duration-300"
                aria-labelledby="sub-menu-academy"
              >
                <li>
                  <a href="#">
                    <span class="icon-[tabler--books] size-5" />
                    Courses
                  </a>
                </li>
                <li>
                  <a href="#">
                    <span class="icon-[tabler--list-details] size-5" />
                    Course details
                  </a>
                </li>
                <li class="space-y-0.5">
                  <a
                    id="sub-menu-academy-stats"
                    class="collapse-toggle collapse-open:bg-base-content/10"
                    data-collapse="#sub-menu-academy-stats-collapse"
                  >
                    <span class="icon-[tabler--chart-bar] size-5" />
                    Stats
                    <span
                      class="icon-[tabler--chevron-down] collapse-open:rotate-180 size-4"
                    />
                  </a>
                  <ul
                    id="sub-menu-academy-stats-collapse"
                    class="collapse hidden w-auto space-y-0.5 overflow-hidden transition-[height] duration-300"
                    aria-labelledby="sub-menu-academy-stats"
                  >
                    <li>
                      <a href="#">
                        <span class="icon-[tabler--chart-donut] size-5" />
                        Goals
                      </a>
                    </li>
                  </ul>
                </li>
              </ul>
            </li>
          </ul>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--settings] size-5" />
            Settings
          </a>
        </li>
        <div class="divider text-base-content/50 py-6 after:border-0">
          Account
        </div>
        <li>
          <a href="#">
            <span class="icon-[tabler--login] size-5" />
            Sign In
          </a>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--logout-2] size-5" />
            Sign Out
          </a>
        </li>
        <div class="divider text-base-content/50 py-6 after:border-0">
          Miscellaneous
        </div>
        <li>
          <a href="#">
            <span class="icon-[tabler--users-group] size-5" />
            Support
          </a>
        </li>
        <li>
          <a href="#">
            <span class="icon-[tabler--files] size-5" />
            Documentation
          </a>
        </li>
      </ul>
    </div>
  </aside>
</template>

<script setup lang="ts">
import type { HSOverlay } from 'flyonui/flyonui'

defineProps<{ title?: string; label?: string }>()

defineEmits(['open', 'close'])

const id = useId()

const open = defineModel<boolean>('open', { default: true })

const { t } = useI18n()

const sidebarRef = useTemplateRef('sidebarRef')

const modal = ref<HSOverlay>()

watch(open, async () => {
  if (open.value) {
    await modal.value?.open()
  } else {
    await modal.value?.close(true)
  }
})

onMounted(async () => {
  if (!sidebarRef.value) return

  modal.value = new window.HSOverlay(sidebarRef.value, {
    isClosePrev: true,
  })

  modal.value.on('close', () => {
    open.value = false
  })
})
</script>

<i18n lang="yaml">
de:
  close: Schließen

en:
  close: Close
</i18n>

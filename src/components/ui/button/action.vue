<template>
  <div class="fixed z-10">
    <div
      class="dropdown relative inline-flex [--placement:top] [--strategy:absolute]"
    >
      <button
        :id
        class="dropdown-toggle btn btn-primary btn-lg btn-square dropdown-open:rotate-45"
        aria-haspopup="menu"
        aria-expanded="false"
        aria-label="Menu"
      >
        <Icon
          :name="icon"
          size="46"
        />
      </button>

      <div
        class="dropdown-menu dropdown-open:opacity-100 hidden min-w-60 rtl:left-0 bg-transparent"
        role="menu"
        aria-orientation="vertical"
        :aria-labelledby="id"
      >
        <ul
          class="dropdown-open:ease-in dropdown-open:translate-x-0 -translate-x-1 rtl:translate-x-1 transition duration-300 ease-out"
          data-dropdown-transition
        >
          <li
            v-for="link in menu"
            class="dropdown-item hover:bg-transparent"
          >
            <NuxtLinkLocale
              v-if="link.to"
              :to="link.to"
              class="btn btn-primary flex items-center no-underline rounded-lg flex-nowrap"
            >
              <Icon
                v-if="link.icon"
                :name="link.icon"
                class="me-3"
              />
              {{ link.label }}
            </NuxtLinkLocale>

            <button
              v-else
              @click="link.action"
              class="link hover:link-primary flex items-center no-underline w-full"
            >
              <Icon
                v-if="link.icon"
                :name="link.icon"
                class="me-3"
              />
              {{ link.label }}
            </button>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { IActionMenuItem } from './types';

defineProps({
  menu: {
    type: Array as PropType<IActionMenuItem[]>,
  },
  icon: {
    type: String,
    default: 'mdi:plus',
  },
});

const id = useId();
</script>

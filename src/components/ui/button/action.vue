<template>
  <div class="z-10 pointer-events-auto">
    <div
      class="dropdown relative inline-flex [--placement:top] [--strategy:absolute]"
    >
      <button
        :id
        class="dropdown-toggle btn btn-primary btn-xl btn-square dropdown-open:rotate-45 transition-transform"
        aria-haspopup="menu"
        aria-expanded="false"
        aria-label="Menu"
      >
        <Icon
          :name="icon"
          class="size-11 shrink-0"
        />
      </button>

      <ul
        class="dropdown-menu dropdown-open:opacity-100 hidden min-w-60 bg-transparent shadow-none"
        data-dropdown-transition
        role="menu"
        aria-orientation="vertical"
        :aria-labelledby="id"
      >
        <li
          v-for="link in menu"
          class="dropdown-item hover:bg-transparent px-0 py-1"
        >
          <NuxtLinkLocale
            v-if="link.to"
            :to="link.to"
            class="btn btn-primary flex items-center no-underline rounded-lg flex-nowrap w-full"
          >
            <Icon
              v-if="link.icon"
              :name="link.icon"
              class="me-3"
            />
            {{ te(link.label) ? t(link.label) : link.label }}
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
</template>

<script setup lang="ts">
import type { IActionMenuItem } from './types'

defineProps({
  menu: {
    type: Array as PropType<IActionMenuItem[]>,
  },
  icon: {
    type: String,
    default: 'mdi:plus',
  },
})

const id = useId()

const { t, te } = useI18n()
</script>

<style lang="css" scoped>
@keyframes fadeInStagger {
  from {
    opacity: 0;
    transform: translateY(15px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 2. Die Listenelemente sind standardmäßig unsichtbar, damit sie nicht aufblitzen */
.stagger-menu li {
  opacity: 0;
}

/* 3. Wenn das Menü geöffnet wird, weise die Animation zu */
:global(.dropdown-open) .stagger-menu li {
  animation-name: fadeInStagger;
  animation-duration: 0.4s;
  animation-timing-function: ease-out;

  /* SEHR WICHTIG: Sorgt dafür, dass die Elemente nach der Animation sichtbar bleiben (den Zustand von 'to' beibehalten) */
  animation-fill-mode: forwards;

  /* Die individuelle animation-delay wird per :style im Template gesetzt. */
}
</style>

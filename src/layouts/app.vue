<template>
  <div class="flex flex-col w-full h-full overflow-hidden">
    <div ref="headerRef">
      <UPageHeader
        as="header"
        :ui="{
          root: [
            'bg-default border-b border-accented sticky top-0 z-50 pt-2 px-8 h-header',
          ],
          wrapper: [
            'flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4',
          ],
        }"
      >
        <template #title>
          <div class="flex items-center">
            <UiLogoHaexhub class="size-12 shrink-0" />

            <NuxtLinkLocale
              class="link text-base-content link-neutral text-xl font-semibold no-underline flex items-center"
              :to="{ name: 'desktop' }"
            >
              <UiTextGradient class="text-nowrap">
                {{ currentVaultName }}
              </UiTextGradient>
            </NuxtLinkLocale>
          </div>
        </template>

        <template #links>
          <UButton
            color="neutral"
            variant="outline"
            :block="isSmallScreen"
            @click="isOverviewMode = !isOverviewMode"
            icon="i-bi-person-workspace"
            size="lg"
          >
          </UButton>
          <UButton
            color="neutral"
            variant="outline"
            :block="isSmallScreen"
            @click="showWindowOverview = !showWindowOverview"
            icon="i-heroicons-squares-2x2"
            size="lg"
          >
            <template #trailing v-if="openWindowsCount > 0">
              <UBadge
                :label="openWindowsCount.toString()"
                color="primary"
                size="xs"
              />
            </template>
          </UButton>
          <HaexExtensionLauncher :block="isSmallScreen" />
        </template>
      </UPageHeader>
    </div>

    <main class="flex-1 overflow-hidden bg-elevated">
      <NuxtPage />
    </main>
  </div>
</template>

<script setup lang="ts">
const { currentVaultName } = storeToRefs(useVaultStore())

const { isSmallScreen } = storeToRefs(useUiStore())

const { isOverviewMode } = storeToRefs(useWorkspaceStore())

const { showWindowOverview, openWindowsCount } = storeToRefs(
  useWindowManagerStore(),
)
</script>

<i18n lang="yaml">
de:
  vault:
    close: Vault schließen

  sidebar:
    close: Sidebar ausblenden
    show: Sidebar anzeigen

  search:
    label: Suche
en:
  vault:
    close: Close vault
  sidebar:
    close: close sidebar
    show: show sidebar

  search:
    label: Search
</i18n>

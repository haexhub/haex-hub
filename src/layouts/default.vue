<template>
  <div class="w-full h-dvh flex flex-col">
    <UPageHeader
      ref="headerEl"
      as="header"
      :ui="{
        root: ['px-8 py-0'],
        wrapper: ['flex flex-row items-center justify-between gap-4'],
      }"
    >
      <template #default>
        <div class="flex justify-between items-center py-1">
          <div>
            <!-- <NuxtLinkLocale
                class="link text-base-content link-neutral text-xl font-semibold no-underline flex items-center"
                :to="{ name: 'desktop' }"
              >
                <UiTextGradient class="text-nowrap">
                  {{ currentVaultName }}
                </UiTextGradient>
              </NuxtLinkLocale> -->
            <UiButton
              v-if="currentVaultId"
              color="neutral"
              variant="outline"
              icon="i-bi-person-workspace"
              size="lg"
              :tooltip="t('workspaces.label')"
              @click="isOverviewMode = !isOverviewMode"
            />
          </div>

          <div>
            <div v-if="!currentVaultId">
              <UiDropdownLocale @select="onSelectLocale" />
            </div>
            <div
              v-else
              class="flex flex-row gap-2"
            >
              <UButton
                v-if="openWindowsCount > 0"
                color="primary"
                variant="outline"
                size="lg"
                @click="showWindowOverview = !showWindowOverview"
              >
                {{ openWindowsCount }}
              </UButton>
              <HaexExtensionLauncher />
            </div>
          </div>
        </div>
      </template>
    </UPageHeader>

    <main class="overflow-hidden relative bg-elevated h-full">
      <slot />
    </main>

    <!-- Workspace Drawer -->
    <HaexWorkspaceDrawer />
  </div>
</template>

<script setup lang="ts">
import type { Locale } from 'vue-i18n'

const { t, setLocale } = useI18n()
const onSelectLocale = async (locale: Locale) => {
  await setLocale(locale)
}

const { currentVaultId } = storeToRefs(useVaultStore())
const { showWindowOverview, openWindowsCount } = storeToRefs(
  useWindowManagerStore(),
)

const { isOverviewMode } = storeToRefs(useWorkspaceStore())

// Measure header height and store it in UI store
const headerEl = useTemplateRef('headerEl')
const { height } = useElementSize(headerEl)
const uiStore = useUiStore()

watch(height, (newHeight) => {
  uiStore.headerHeight = newHeight
})
</script>

<i18n lang="yaml">
de:
  search:
    label: Suche
  workspaces:
    label: Workspaces
en:
  search:
    label: Search
  workspaces:
    label: Workspaces
</i18n>

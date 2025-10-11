<template>
  <UCard
    :ui="{
      root: 'hover:shadow-lg transition-shadow duration-200 cursor-pointer',
      body: 'flex flex-col gap-3',
    }"
    @click="$emit('click')"
  >
    <div class="flex items-start gap-4">
      <!-- Icon -->
      <div class="flex-shrink-0">
        <div
          v-if="extension.icon"
          class="w-16 h-16 rounded-lg bg-primary/10 flex items-center justify-center"
        >
          <UIcon
            :name="extension.icon"
            class="w-10 h-10 text-primary"
          />
        </div>
        <div
          v-else
          class="w-16 h-16 rounded-lg bg-gray-200 dark:bg-gray-700 flex items-center justify-center"
        >
          <UIcon
            name="i-heroicons-puzzle-piece"
            class="w-10 h-10 text-gray-400"
          />
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 min-w-0">
        <div class="flex items-start justify-between gap-2">
          <div class="flex-1 min-w-0">
            <h3 class="text-lg font-semibold truncate">
              {{ extension.name }}
            </h3>
            <p
              v-if="extension.author"
              class="text-sm text-gray-500 dark:text-gray-400"
            >
              {{ t('by') }} {{ extension.author }}
            </p>
          </div>
          <UBadge
            :label="extension.version"
            color="neutral"
            variant="subtle"
          />
        </div>

        <p
          v-if="extension.description"
          class="text-sm text-gray-600 dark:text-gray-300 mt-2 line-clamp-2"
        >
          {{ extension.description }}
        </p>

        <!-- Stats and Status -->
        <div
          class="flex items-center gap-4 mt-3 text-sm text-gray-500 dark:text-gray-400"
        >
          <div
            v-if="isInstalled"
            class="flex items-center gap-1 text-success font-medium"
          >
            <UIcon name="i-heroicons-check-circle-solid" />
            <span>{{ t('installed') }}</span>
          </div>
          <div
            v-if="extension.downloads"
            class="flex items-center gap-1"
          >
            <UIcon name="i-heroicons-arrow-down-tray" />
            <span>{{ formatNumber(extension.downloads) }}</span>
          </div>
          <div
            v-if="extension.rating"
            class="flex items-center gap-1"
          >
            <UIcon name="i-heroicons-star-solid" />
            <span>{{ extension.rating }}</span>
          </div>
          <div
            v-if="extension.verified"
            class="flex items-center gap-1 text-green-600 dark:text-green-400"
          >
            <UIcon name="i-heroicons-check-badge-solid" />
            <span>{{ t('verified') }}</span>
          </div>
        </div>

        <!-- Tags -->
        <div
          v-if="extension.tags?.length"
          class="flex flex-wrap gap-1 mt-2"
        >
          <UBadge
            v-for="tag in extension.tags.slice(0, 3)"
            :key="tag"
            :label="tag"
            size="xs"
            color="primary"
            variant="soft"
          />
        </div>
      </div>
    </div>

    <!-- Actions -->
    <template #footer>
      <div class="flex items-center justify-between gap-2">
        <UButton
          :label="isInstalled ? t('installed') : t('install')"
          :color="isInstalled ? 'neutral' : 'primary'"
          :disabled="isInstalled"
          :icon="
            isInstalled ? 'i-heroicons-check' : 'i-heroicons-arrow-down-tray'
          "
          size="sm"
          @click.stop="$emit('install')"
        />
        <UButton
          :label="t('details')"
          color="neutral"
          variant="ghost"
          size="sm"
          @click.stop="$emit('details')"
        />
      </div>
    </template>
  </UCard>
</template>

<script setup lang="ts">
interface MarketplaceExtension {
  id: string
  name: string
  version: string
  author?: string
  description?: string
  icon?: string
  downloads?: number
  rating?: number
  verified?: boolean
  tags?: string[]
  downloadUrl?: string
}

defineProps<{
  extension: MarketplaceExtension
  isInstalled?: boolean
}>()

defineEmits(['click', 'install', 'details'])

const { t } = useI18n()

const formatNumber = (num: number) => {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toString()
}
</script>

<i18n lang="yaml">
de:
  by: von
  install: Installieren
  installed: Installiert
  details: Details
  verified: Verifiziert
en:
  by: by
  install: Install
  installed: Installed
  details: Details
  verified: Verified
</i18n>

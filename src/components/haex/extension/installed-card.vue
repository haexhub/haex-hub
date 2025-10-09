<template>
  <UCard
    :ui="{
      root: 'hover:shadow-lg transition-shadow duration-200 cursor-pointer',
      body: 'flex flex-col gap-3',
    }"
    @click="$emit('open')"
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

        <!-- Installed Badge -->
        <div class="flex items-center gap-2 mt-3">
          <UBadge
            :label="t('installed')"
            color="success"
            variant="subtle"
          >
            <template #leading>
              <UIcon name="i-heroicons-check-circle" />
            </template>
          </UBadge>
          <UBadge
            v-if="extension.enabled"
            :label="t('enabled')"
            color="primary"
            variant="soft"
          />
          <UBadge
            v-else
            :label="t('disabled')"
            color="neutral"
            variant="soft"
          />
        </div>
      </div>
    </div>

    <!-- Actions -->
    <template #footer>
      <div class="flex items-center justify-between gap-2">
        <UButton
          :label="t('open')"
          color="primary"
          icon="i-heroicons-arrow-right"
          size="sm"
          @click.stop="$emit('open')"
        />
        <div class="flex gap-2">
          <UButton
            :label="t('settings')"
            color="neutral"
            variant="ghost"
            icon="i-heroicons-cog-6-tooth"
            size="sm"
            @click.stop="$emit('settings')"
          />
          <UButton
            :label="t('remove')"
            color="error"
            variant="ghost"
            icon="i-heroicons-trash"
            size="sm"
            @click.stop="$emit('remove')"
          />
        </div>
      </div>
    </template>
  </UCard>
</template>

<script setup lang="ts">
interface InstalledExtension {
  id: string
  name: string
  version: string
  author?: string
  description?: string
  icon?: string
  enabled?: boolean
}

defineProps<{
  extension: InstalledExtension
}>()

defineEmits(['open', 'settings', 'remove'])

const { t } = useI18n()
</script>

<i18n lang="yaml">
de:
  by: von
  installed: Installiert
  enabled: Aktiviert
  disabled: Deaktiviert
  open: Öffnen
  settings: Einstellungen
  remove: Entfernen
en:
  by: by
  installed: Installed
  enabled: Enabled
  disabled: Disabled
  open: Open
  settings: Settings
  remove: Remove
</i18n>

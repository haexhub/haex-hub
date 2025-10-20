<template>
  <div class="w-full h-full flex flex-col bg-white dark:bg-gray-900">
    <!-- Marketplace Header -->
    <div
      class="flex-shrink-0 border-b border-gray-200 dark:border-gray-700 p-6"
    >
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
        Extension Marketplace
      </h1>
      <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
        Discover and install extensions for HaexHub
      </p>
    </div>

    <!-- Search Bar -->
    <div
      class="flex-shrink-0 border-b border-gray-200 dark:border-gray-700 p-4"
    >
      <UInput
        icon="i-heroicons-magnifying-glass"
        size="lg"
        placeholder="Search extensions..."
        class="w-full"
      />
    </div>

    <!-- Marketplace Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-4xl space-y-6">
        <!-- Featured Extensions -->
        <section>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Featured Extensions
          </h2>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- Example Extension Card -->
            <div
              class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 hover:shadow-lg transition-shadow cursor-pointer"
            >
              <div class="flex items-start gap-4">
                <div
                  class="w-12 h-12 rounded-lg bg-primary-500 flex items-center justify-center flex-shrink-0"
                >
                  <UIcon
                    name="i-heroicons-puzzle-piece"
                    class="w-6 h-6 text-white"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <h3
                    class="font-semibold text-gray-900 dark:text-white truncate"
                  >
                    Example Extension
                  </h3>
                  <p
                    class="text-sm text-gray-600 dark:text-gray-400 line-clamp-2 mt-1"
                  >
                    A powerful extension for HaexHub
                  </p>
                  <div class="flex items-center gap-2 mt-2">
                    <span class="text-xs text-gray-500 dark:text-gray-500"
                      >v1.0.0</span
                    >
                    <span class="text-xs text-gray-500 dark:text-gray-500"
                      >•</span
                    >
                    <span class="text-xs text-gray-500 dark:text-gray-500"
                      >1.2k downloads</span
                    >
                  </div>
                </div>
                <UButton
                  label="Install"
                  size="sm"
                  color="primary"
                />
              </div>
            </div>

            <!-- Placeholder for more extensions -->
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 opacity-50">
              <div class="flex items-start gap-4">
                <div
                  class="w-12 h-12 rounded-lg bg-gray-400 flex items-center justify-center flex-shrink-0"
                >
                  <UIcon
                    name="i-heroicons-puzzle-piece"
                    class="w-6 h-6 text-white"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <h3
                    class="font-semibold text-gray-900 dark:text-white truncate"
                  >
                    More extensions coming soon...
                  </h3>
                  <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    Check back later for more extensions
                  </p>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Categories -->
        <section>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Categories
          </h2>
          <div class="flex flex-wrap gap-2">
            <UBadge
              label="Productivity"
              color="primary"
              variant="soft"
              size="lg"
            />
            <UBadge
              label="Development"
              color="secondary"
              variant="soft"
              size="lg"
            />
            <UBadge
              label="Security"
              color="error"
              variant="soft"
              size="lg"
            />
            <UBadge
              label="Utilities"
              color="secondary"
              variant="soft"
              size="lg"
            />
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// Marketplace component - placeholder implementation
</script>

<template>
  <div class="w-full h-full flex flex-col bg-white dark:bg-gray-900">
    <!-- Settings Header -->
    <div
      class="flex-shrink-0 border-b border-gray-200 dark:border-gray-700 p-6"
    >
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Settings</h1>
      <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
        Manage your HaexHub preferences and configuration
      </p>
    </div>

    <!-- Settings Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-2xl space-y-6">
        <!-- General Section -->
        <section>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            General
          </h2>
          <div class="space-y-4 bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="font-medium text-gray-900 dark:text-white">Theme</p>
                <p class="text-sm text-gray-600 dark:text-gray-400">
                  Choose your preferred theme
                </p>
              </div>
              <UButton
                label="Auto"
                variant="outline"
              />
            </div>
            <div class="flex items-center justify-between">
              <div>
                <p class="font-medium text-gray-900 dark:text-white">
                  Language
                </p>
                <p class="text-sm text-gray-600 dark:text-gray-400">
                  Select your language
                </p>
              </div>
              <UButton
                label="English"
                variant="outline"
              />
            </div>
          </div>
        </section>

        <!-- Privacy Section -->
        <section>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Privacy & Security
          </h2>
          <div class="space-y-4 bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="font-medium text-gray-900 dark:text-white">
                  Auto-lock
                </p>
                <p class="text-sm text-gray-600 dark:text-gray-400">
                  Lock vault after inactivity
                </p>
              </div>
            </div>
          </div>
        </section>

        <!-- About Section -->
        <section>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            About
          </h2>
          <div class="space-y-2 bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div class="flex justify-between">
              <span class="text-sm text-gray-600 dark:text-gray-400"
                >Version</span
              >
              <span class="text-sm font-medium text-gray-900 dark:text-white"
                >0.1.0</span
              >
            </div>
            <div class="flex justify-between">
              <span class="text-sm text-gray-600 dark:text-gray-400"
                >Platform</span
              >
              <span class="text-sm font-medium text-gray-900 dark:text-white"
                >Tauri + Vue</span
              >
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// Settings component - placeholder implementation
</script>

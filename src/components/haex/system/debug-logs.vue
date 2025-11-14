<template>
  <div class="w-full h-full bg-default flex flex-col">
    <!-- Header with controls -->
    <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
      <div class="flex items-center gap-2">
        <UIcon
          name="i-heroicons-bug-ant"
          class="w-5 h-5"
        />
        <h2 class="text-lg font-semibold">
          Debug Logs
        </h2>
        <span class="text-xs text-gray-500">
          {{ logs.length }} logs
        </span>
      </div>
      <div class="flex gap-2">
        <UButton
          :label="allCopied ? 'Copied!' : 'Copy All'"
          :color="allCopied ? 'success' : 'primary'"
          size="sm"
          @click="copyAllLogs"
        />
        <UButton
          label="Clear Logs"
          color="error"
          size="sm"
          @click="clearLogs"
        />
      </div>
    </div>

    <!-- Filter Buttons -->
    <div class="flex gap-2 p-4 border-b border-gray-200 dark:border-gray-700 overflow-x-auto">
      <UButton
        v-for="level in ['all', 'log', 'info', 'warn', 'error', 'debug']"
        :key="level"
        :label="level"
        :color="filter === level ? 'primary' : 'neutral'"
        size="sm"
        @click="filter = level as any"
      />
    </div>

    <!-- Logs Container -->
    <div
      ref="logsContainer"
      class="flex-1 overflow-y-auto p-4 space-y-2 font-mono text-xs"
    >
      <div
        v-for="(log, index) in filteredLogs"
        :key="index"
        :class="[
          'p-3 rounded-lg border-l-4 relative group',
          log.level === 'error'
            ? 'bg-red-50 dark:bg-red-950/30 border-red-500'
            : log.level === 'warn'
              ? 'bg-yellow-50 dark:bg-yellow-950/30 border-yellow-500'
              : log.level === 'info'
                ? 'bg-blue-50 dark:bg-blue-950/30 border-blue-500'
                : log.level === 'debug'
                  ? 'bg-purple-50 dark:bg-purple-950/30 border-purple-500'
                  : 'bg-gray-50 dark:bg-gray-800 border-gray-400',
        ]"
      >
        <!-- Copy Button -->
        <button
          class="absolute top-2 right-2 p-1.5 rounded bg-white dark:bg-gray-700 shadow-sm hover:bg-gray-100 dark:hover:bg-gray-600 active:scale-95 transition-all"
          @click="copyLogToClipboard(log)"
        >
          <UIcon
            :name="copiedIndex === index ? 'i-heroicons-check' : 'i-heroicons-clipboard-document'"
            :class="[
              'w-4 h-4',
              copiedIndex === index ? 'text-green-500' : ''
            ]"
          />
        </button>

        <div class="flex items-start gap-2 mb-1">
          <span class="text-gray-500 dark:text-gray-400 text-[10px] shrink-0">
            {{ log.timestamp }}
          </span>
          <span
            :class="[
              'font-semibold text-[10px] uppercase shrink-0',
              log.level === 'error'
                ? 'text-red-600 dark:text-red-400'
                : log.level === 'warn'
                  ? 'text-yellow-600 dark:text-yellow-400'
                  : log.level === 'info'
                    ? 'text-blue-600 dark:text-blue-400'
                    : log.level === 'debug'
                      ? 'text-purple-600 dark:text-purple-400'
                      : 'text-gray-600 dark:text-gray-400',
            ]"
          >
            {{ log.level }}
          </span>
        </div>
        <pre class="whitespace-pre-wrap break-words text-gray-900 dark:text-gray-100 pr-8">{{ log.message }}</pre>
      </div>

      <div
        v-if="filteredLogs.length === 0"
        class="text-center text-gray-500 py-8"
      >
        <UIcon
          name="i-heroicons-document-text"
          class="w-12 h-12 mx-auto mb-2 opacity-50"
        />
        <p>No logs to display</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { globalConsoleLogs } from '~/plugins/console-interceptor'
import type { ConsoleLog } from '~/plugins/console-interceptor'

const filter = ref<'all' | 'log' | 'info' | 'warn' | 'error' | 'debug'>('all')
const logsContainer = ref<HTMLDivElement>()
const copiedIndex = ref<number | null>(null)
const allCopied = ref(false)

const { $clearConsoleLogs } = useNuxtApp()
const { copy } = useClipboard()

const logs = computed(() => globalConsoleLogs.value)

const filteredLogs = computed(() => {
  if (filter.value === 'all') {
    return logs.value
  }
  return logs.value.filter((log) => log.level === filter.value)
})

const clearLogs = () => {
  if ($clearConsoleLogs) {
    $clearConsoleLogs()
  }
}

const copyLogToClipboard = async (log: ConsoleLog) => {
  const text = `[${log.timestamp}] [${log.level.toUpperCase()}] ${log.message}`
  await copy(text)

  // Find the index in filteredLogs for visual feedback
  const index = filteredLogs.value.indexOf(log)
  copiedIndex.value = index

  // Reset after 2 seconds
  setTimeout(() => {
    copiedIndex.value = null
  }, 2000)
}

const copyAllLogs = async () => {
  const allLogsText = filteredLogs.value
    .map((log) => `[${log.timestamp}] [${log.level.toUpperCase()}] ${log.message}`)
    .join('\n')

  await copy(allLogsText)
  allCopied.value = true

  // Reset after 2 seconds
  setTimeout(() => {
    allCopied.value = false
  }, 2000)
}

// Auto-scroll to bottom when new logs arrive
watch(
  () => logs.value.length,
  () => {
    nextTick(() => {
      if (logsContainer.value) {
        logsContainer.value.scrollTop = logsContainer.value.scrollHeight
      }
    })
  },
  { immediate: true }
)
</script>

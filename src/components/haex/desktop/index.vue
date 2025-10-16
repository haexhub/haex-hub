<template>
  <div
    class="w-full h-full relative overflow-hidden bg-gradient-to-br from-blue-50 to-blue-100 dark:from-gray-900 dark:to-gray-800"
  >
    <!-- Dropzones (only visible during drag) -->
    <Transition name="slide-down">
      <div
        v-if="isDragging"
        class="absolute top-0 left-0 right-0 flex gap-2 p-4 z-50"
      >
        <!-- Remove from Desktop Dropzone -->
        <div
          ref="removeDropzoneEl"
          class="flex-1 h-20 flex items-center justify-center gap-2 rounded-lg border-2 border-dashed transition-all"
          :class="
            isOverRemoveZone
              ? 'bg-orange-500/20 border-orange-500 dark:bg-orange-400/20 dark:border-orange-400'
              : 'border-orange-500/50 dark:border-orange-400/50'
          "
        >
          <Icon
            name="i-heroicons-x-mark"
            class="w-6 h-6"
            :class="
              isOverRemoveZone
                ? 'text-orange-700 dark:text-orange-300'
                : 'text-orange-600 dark:text-orange-400'
            "
          />
          <span
            class="font-semibold"
            :class="
              isOverRemoveZone
                ? 'text-orange-700 dark:text-orange-300'
                : 'text-orange-600 dark:text-orange-400'
            "
          >
            Von Desktop entfernen
          </span>
        </div>

        <!-- Uninstall Dropzone -->
        <div
          ref="uninstallDropzoneEl"
          class="flex-1 h-20 flex items-center justify-center gap-2 rounded-lg border-2 border-dashed transition-all"
          :class="
            isOverUninstallZone
              ? 'bg-red-500/20 border-red-500 dark:bg-red-400/20 dark:border-red-400'
              : 'border-red-500/50 dark:border-red-400/50'
          "
        >
          <Icon
            name="i-heroicons-trash"
            class="w-6 h-6"
            :class="
              isOverUninstallZone
                ? 'text-red-700 dark:text-red-300'
                : 'text-red-600 dark:text-red-400'
            "
          />
          <span
            class="font-semibold"
            :class="
              isOverUninstallZone
                ? 'text-red-700 dark:text-red-300'
                : 'text-red-600 dark:text-red-400'
            "
          >
            Deinstallieren
          </span>
        </div>
      </div>
    </Transition>

    <HaexDesktopIcon
      v-for="item in desktopItemIcons"
      :key="item.id"
      :id="item.id"
      :item-type="item.itemType"
      :reference-id="item.referenceId"
      :initial-x="item.positionX"
      :initial-y="item.positionY"
      :label="item.label"
      :icon="item.icon"
      @position-changed="handlePositionChanged"
      @open="handleOpen"
      @drag-start="handleDragStart"
      @drag-end="handleDragEnd"
      @uninstall="handleUninstall"
    />
  </div>
</template>

<script setup lang="ts">
const desktopStore = useDesktopStore()
const extensionsStore = useExtensionsStore()
const router = useRouter()

const { desktopItems } = storeToRefs(desktopStore)
const { availableExtensions } = storeToRefs(extensionsStore)

// Drag state
const isDragging = ref(false)
const currentDraggedItemId = ref<string>()
const currentDraggedItemType = ref<string>()
const currentDraggedReferenceId = ref<string>()

// Dropzone refs
const removeDropzoneEl = ref<HTMLElement>()
const uninstallDropzoneEl = ref<HTMLElement>()

// Setup dropzones with VueUse
const { isOverDropZone: isOverRemoveZone } = useDropZone(removeDropzoneEl, {
  onDrop: () => {
    if (currentDraggedItemId.value) {
      handleRemoveFromDesktop(currentDraggedItemId.value)
    }
  },
})

const { isOverDropZone: isOverUninstallZone } = useDropZone(uninstallDropzoneEl, {
  onDrop: () => {
    if (currentDraggedItemType.value && currentDraggedReferenceId.value) {
      handleUninstall(currentDraggedItemType.value, currentDraggedReferenceId.value)
    }
  },
})

interface DesktopItemIcon extends IDesktopItem {
  label: string
  icon?: string
}

const desktopItemIcons = computed<DesktopItemIcon[]>(() => {
  return desktopItems.value.map((item) => {
    if (item.itemType === 'extension') {
      const extension = availableExtensions.value.find(
        (ext) => ext.id === item.referenceId,
      )

      return {
        ...item,
        label: extension?.name || 'Unknown',
        icon: extension?.icon || '',
      }
    }

    if (item.itemType === 'file') {
      // Für später: file handling
      return {
        ...item,
        label: item.referenceId,
        icon: undefined,
      }
    }

    if (item.itemType === 'folder') {
      // Für später: folder handling
      return {
        ...item,
        label: item.referenceId,
        icon: undefined,
      }
    }

    return {
      ...item,
      label: item.referenceId,
      icon: undefined,
    }
  })
})

const handlePositionChanged = async (id: string, x: number, y: number) => {
  try {
    await desktopStore.updateDesktopItemPositionAsync(id, x, y)
  } catch (error) {
    console.error('Fehler beim Speichern der Position:', error)
  }
}

const localePath = useLocalePath()

const handleOpen = (itemType: string, referenceId: string) => {
  if (itemType === 'extension') {
    router.push(
      localePath({
        name: 'extension',
        params: { extensionId: referenceId },
      })
    )
  }
  // Für später: file und folder handling
}

const handleDragStart = (id: string, itemType: string, referenceId: string) => {
  isDragging.value = true
  currentDraggedItemId.value = id
  currentDraggedItemType.value = itemType
  currentDraggedReferenceId.value = referenceId
}

const handleDragEnd = () => {
  isDragging.value = false
  currentDraggedItemId.value = undefined
  currentDraggedItemType.value = undefined
  currentDraggedReferenceId.value = undefined
}

const handleUninstall = async (itemType: string, referenceId: string) => {
  if (itemType === 'extension') {
    try {
      const extension = availableExtensions.value.find((ext) => ext.id === referenceId)
      if (extension) {
        await extensionsStore.removeExtensionAsync(
          extension.publicKey,
          extension.name,
          extension.version,
        )
        // Reload extensions after uninstall
        await extensionsStore.loadExtensionsAsync()
      }
    } catch (error) {
      console.error('Fehler beim Deinstallieren:', error)
    }
  }
  // Für später: file und folder handling
}

onMounted(async () => {
  await desktopStore.loadDesktopItemsAsync()
})
</script>

<style scoped>
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.3s ease;
}

.slide-down-enter-from {
  opacity: 0;
  transform: translateY(-100%);
}

.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-100%);
}
</style>

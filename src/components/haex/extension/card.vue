<template>
  <div
    class="card border-4 shadow-md shadow-accent h-48 w-48 overflow-hidden hover:shadow-xl transition-shadow"
    v-bind="$attrs"
  >
    <div class="absolute top-2 right-2">
      <UDropdownMenu>
        <UiButton icon="mdi:dots-vertical" />
      </UDropdownMenu>
    </div>

    <div class="card-header">
      <h5
        v-if="name"
        class="card-title"
      >
        {{ name }}
      </h5>
    </div>

    <div
      class="card-body relative cursor-pointer"
      @click="
        navigateTo(
          useLocalePath()({
            name: 'haexExtension',
            params: { extensionId: id },
          }),
        )
      "
    >
      <!-- <slot />
      <div class="card-actions" v-if="$slots.action">
        <slot name="action" />
      </div> -->
      hier klicken
      <div
        class="size-20 absolute bottom-2 right-2"
        v-html="icon"
      />
    </div>

    <!-- <div class="card-footer">

    </div> -->
  </div>

  <HaexExtensionDialogRemove
    v-model:open="showRemoveDialog"
    :extension
    @confirm="removeExtensionAsync"
  />
</template>

<script setup lang="ts">
import type { IHaexHubExtension } from '~/types/haexhub'
const emit = defineEmits(['close', 'submit', 'remove'])

const extension = defineProps<IHaexHubExtension>()

const { escape, enter } = useMagicKeys()

watchEffect(async () => {
  if (escape?.value) {
    await nextTick()
    emit('close')
  }
})

watchEffect(async () => {
  if (enter?.value) {
    await nextTick()
    emit('submit')
  }
})

const showRemoveDialog = ref(false)
const { add } = useToast()
const { t } = useI18n()
const extensionStore = useExtensionsStore()

const removeExtensionAsync = async () => {
  if (!extension?.id || !extension?.version) {
    add({
      color: 'error',
      description: 'Erweiterung kann nicht gelöscht werden',
    })
    return
  }

  try {
    await extensionStore.removeExtensionAsync(
      extension.publicKey,
      extension.name,
      extension.version,
    )
    await extensionStore.loadExtensionsAsync()

    add({
      color: 'success',
      title: t('extension.remove.success.title', {
        extensionName: extension.name,
      }),
      description: t('extension.remove.success.text', {
        extensionName: extension.name,
      }),
    })
  } catch (error) {
    add({
      color: 'error',
      title: t('extension.remove.error.title'),
      description: t('extension.remove.error.text', {
        error: JSON.stringify(error),
      }),
    })
  }
}
</script>

<i18n lang="yaml">
de:
  remove: Löschen
  extension:
    remove:
      success:
        text: 'Erweiterung {extensionName} wurde erfolgreich entfernt'
        title: '{extensionName} entfernt'
      error:
        text: "Erweiterung {extensionName} konnte nicht entfernt werden. \n {error}"
        title: 'Fehler beim Entfernen von {extensionName}'

en:
  remove: Remove
  extension:
    remove:
      success:
        text: 'Extension {extensionName} was removed'
        title: '{extensionName} removed'
      error:
        text: "Extension {extensionName} couldn't be removed. \n {error}"
        title: 'Exception during uninstall {extensionName}'
</i18n>

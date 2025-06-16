<template>
  <div>
    <HaexPassGroup
      v-model="currentGroup"
      mode="edit"
      @close="onClose"
      @submit="onSaveAsync"
    />
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'passwordGroupEdit',
})

const { t } = useI18n()

const check = ref(false)

const { currentGroup } = storeToRefs(usePasswordGroupStore())

//const group = computed(() => currentGroup.value)

const errors = ref({
  name: [],
  description: [],
})

const onClose = () => {
  useRouter().back()
}

const { add } = useSnackbar()

const onSaveAsync = async () => {
  try {
    check.value = true
    if (!currentGroup.value) return

    console.log('onSave', errors.value)
    if (errors.value.name.length || errors.value.description.length) return

    const { updateAsync } = usePasswordGroupStore()

    await updateAsync(currentGroup.value)

    add({ type: 'success', text: t('change.success') })
    onClose()
  } catch (error) {
    add({ type: 'error', text: t('change.error') })
    console.log(error)
  }
}
</script>

<i18n lang="yaml">
de:
  title: Gruppe ändern
  abort: Abbrechen
  save: Speichern
  name:
    label: Name

  description:
    label: Beschreibung

  change:
    success: Änderung erfolgreich gespeichert
    error: Änderung konnte nicht gespeichert werden

en:
  title: Edit Group
  abort: Abort
  save: Save
  name:
    label: Name

  description:
    label: Description

  change:
    success: Change successfully saved
    error: Change could not be saved
</i18n>

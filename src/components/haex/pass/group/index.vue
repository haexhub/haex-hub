<template>
  <UCard
    v-if="group"
    :ui="{ root: [''] }"
  >
    <template #header>
      <div class="flex items-center gap-2">
        <Icon
          :name="
            mode === 'edit'
              ? 'mdi:folder-edit-outline'
              : 'mdi:folder-plus-outline'
          "
          size="24"
        />
        <span>{{ mode === 'edit' ? t('title.edit') : t('title.create') }}</span>
      </div>
    </template>

    <form class="flex flex-col gap-4 w-full p-4">
      <UiInput
        ref="nameRef"
        v-model="group.name"
        :label="t('name')"
        :placeholder="t('name')"
        :read-only
        autofocus
        @keyup.enter="$emit('submit')"
      />

      <UiInput
        v-model="group.description"
        :label="t('description')"
        :placeholder="t('description')"
        :read-only
        @keyup.enter="$emit('submit')"
      />

      <div class="flex flex-wrap gap-4">
        <!-- <UiSelectIcon
            v-model="group.icon"
            default-icon="mdi:folder-outline"
            :readOnly
          />

          <UiSelectColor
            v-model="group.color"
            :readOnly
          /> -->
      </div>
    </form>
  </UCard>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

const group = defineModel<SelectHaexPasswordsGroups | null>()
const { readOnly = false } = defineProps<{
  readOnly?: boolean
  mode: 'create' | 'edit'
}>()
const emit = defineEmits(['close', 'submit'])

const { t } = useI18n()

const nameRef = useTemplateRef('nameRef')
onStartTyping(() => {
  nameRef.value?.$el.focus()
})

const { escape } = useMagicKeys()

watchEffect(async () => {
  if (escape?.value) {
    await nextTick()
    emit('close')
  }
})
</script>

<i18n lang="yaml">
de:
  name: Name
  description: Beschreibung
  icon: Icon
  color: Farbe
  create: Erstellen
  save: Speichern
  abort: Abbrechen
  title:
    create: Gruppe erstellen
    edit: Gruppe ändern

en:
  name: Name
  description: Description
  icon: Icon
  color: Color
  create: Create
  save: Save
  abort: Abort
  title:
    create: Create group
    edit: Edit group
</i18n>

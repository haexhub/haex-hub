<template>
  <div class="p-1">
    <UiCard
      v-if="group"
      :title="mode === 'edit' ? t('title.edit') : t('title.create')"
      icon="mdi:folder-plus-outline"
      @close="$emit('close')"
      body-class="px-0"
    >
      <form
        class="flex flex-col gap-4 w-full p-4"
        @submit.prevent="$emit('submit')"
      >
        <UiInput
          :label="t('name')"
          :placeholder="t('name')"
          :read_only
          autofocus
          v-model="group.name"
          ref="nameRef"
          @keyup.enter="$emit('submit')"
        />

        <UiInput
          v-model="group.description"
          :label="t('description')"
          :placeholder="t('description')"
          :read_only
          @keyup.enter="$emit('submit')"
        />

        <div class="flex flex-wrap gap-4">
          <UiSelectIcon
            v-model="group.icon"
            default-icon="mdi:folder-outline"
            :read_only
          />

          <UiSelectColor
            v-model="group.color"
            :read_only
          />
        </div>

        <!-- <div class="flex flex-wrap justify-end gap-4">
          <UiButton
            class="btn-error btn-outline flex-1"
            @click="$emit('close')"
          >
            {{ t('abort') }}
            <Icon name="mdi:close" />
          </UiButton>

          <UiButton
            class="btn-primary flex-1"
            @click="$emit('submit')"
          >
            {{ mode === 'create' ? t('create') : t('save') }}
            <Icon name="mdi:check" />
          </UiButton>
        </div> -->
      </form>
    </UiCard>
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

const group = defineModel<SelectHaexPasswordsGroups | null>()
const { read_only = false } = defineProps<{
  read_only?: boolean
  mode: 'create' | 'edit'
}>()
defineEmits(['close', 'submit'])

const { t } = useI18n()

const nameRef = useTemplateRef('nameRef')
onStartTyping(() => {
  nameRef.value?.inputRef?.focus()
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

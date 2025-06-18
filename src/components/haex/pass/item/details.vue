<template>
  <div class="h-full overflow-scroll">
    <form
      class="flex flex-col gap-4 w-full p-4"
      @submit.prevent="$emit('submit')"
    >
      <UiInput
        v-show="!read_only || itemDetails.title"
        :check-input="check"
        :label="t('item.title')"
        :placeholder="t('item.title')"
        :read_only
        :with-copy-button
        autofocus
        ref="titleRef"
        v-model.trim="itemDetails.title"
        @keyup.enter="$emit('submit')"
      />

      <UiInput
        v-show="!read_only || itemDetails.username"
        :check-input="check"
        :label="t('item.username')"
        :placeholder="t('item.username')"
        :with-copy-button
        :read_only
        v-model.trim="itemDetails.username"
      />

      <UiInputPassword
        v-show="!read_only || itemDetails.password"
        :check-input="check"
        :read_only
        :with-copy-button
        v-model.trim="itemDetails.password"
      >
        <template #append>
          <UiDialogPasswordGenerator
            v-if="!read_only"
            class="join-item"
            :password="itemDetails.password"
            v-model="preventClose"
          />
        </template>
      </UiInputPassword>

      <UiInputUrl
        v-show="!read_only || itemDetails.url"
        :label="t('item.url')"
        :placeholder="t('item.url')"
        :read_only
        :with-copy-button
        v-model="itemDetails.url"
      />

      <UiSelectIcon
        v-show="!read_only"
        :default-icon="defaultIcon || 'mdi:key-outline'"
        :read_only
        v-model="itemDetails.icon"
      />

      <UiTextarea
        v-show="!read_only || itemDetails.note"
        v-model="itemDetails.note"
        :label="t('item.note')"
        :placeholder="t('item.note')"
        :read_only
        :with-copy-button
        @keyup.enter.stop
        class="h-52"
      />
    </form>
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsItemDetails } from '~~/src-tauri/database/schemas/vault'

defineProps<{
  defaultIcon?: string | null
  read_only?: boolean
  withCopyButton?: boolean
}>()

defineEmits(['submit'])
const { t } = useI18n()

const itemDetails = defineModel<SelectHaexPasswordsItemDetails>({
  required: true,
})

const preventClose = defineModel<boolean>('preventClose')

const check = defineModel<boolean>('check-input', { default: false })

/* onKeyStroke('escape', (e) => {
  e.stopPropagation()
  e.stopImmediatePropagation()
}) */

const titleRef = useTemplateRef('titleRef')
onStartTyping(() => {
  titleRef.value?.inputRef?.focus()
})
</script>

<i18n lang="yaml">
de:
  item:
    title: Titel
    username: Nutzername
    password: Passwort
    url: Url
    note: Notiz

en:
  item:
    title: Title
    username: Username
    password: Password
    url: Url
    note: Note
</i18n>

<template>
  <div class="h-full overflow-scroll">
    <form
      class="flex flex-col gap-4 w-full p-4"
      @submit.prevent="$emit('submit')"
    >
      <UiInput
        v-show="!readOnly || itemDetails.title"
        ref="titleRef"
        v-model.trim="itemDetails.title"
        :check-input="check"
        :label="t('item.title')"
        :placeholder="t('item.title')"
        :read-only
        :with-copy-button
        autofocus
        @keyup.enter="$emit('submit')"
      />

      <UiInput
        v-show="!readOnly || itemDetails.username"
        v-model.trim="itemDetails.username"
        :check-input="check"
        :label="t('item.username')"
        :placeholder="t('item.username')"
        :with-copy-button
        :read-only
        @keyup.enter="$emit('submit')"
      />

      <UiInputPassword
        v-show="!readOnly || itemDetails.password"
        v-model.trim="itemDetails.password"
        :check-input="check"
        :read-only
        :with-copy-button
        @keyup.enter="$emit('submit')"
      >
        <template #append>
          <!-- <UiDialogPasswordGenerator
            v-if="!readOnly"
            class="join-item"
            :password="itemDetails.password"
            v-model="preventClose"
          /> -->
        </template>
      </UiInputPassword>

      <UiInputUrl
        v-show="!readOnly || itemDetails.url"
        v-model="itemDetails.url"
        :label="t('item.url')"
        :placeholder="t('item.url')"
        :read-only
        :with-copy-button
        @keyup.enter="$emit('submit')"
      />

      <!-- <UiSelectIcon
        v-show="!readOnly"
        :default-icon="defaultIcon || 'mdi:key-outline'"
        :readOnly
        v-model="itemDetails.icon"
      /> -->

      <UiTextarea
        v-show="!readOnly || itemDetails.note"
        v-model="itemDetails.note"
        :label="t('item.note')"
        :placeholder="t('item.note')"
        :readOnly
        :with-copy-button
        @keyup.enter.stop
        color="error"
      />
    </form>
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsItemDetails } from '~~/src-tauri/database/schemas/vault'

defineProps<{
  defaultIcon?: string | null
  readOnly?: boolean
  withCopyButton?: boolean
}>()

defineEmits(['submit'])
const { t } = useI18n()

const itemDetails = defineModel<SelectHaexPasswordsItemDetails>({
  required: true,
})

//const preventClose = defineModel<boolean>('preventClose')

const check = defineModel<boolean>('check-input', { default: false })

/* onKeyStroke('escape', (e) => {
  e.stopPropagation()
  e.stopImmediatePropagation()
}) */

const titleRef = useTemplateRef('titleRef')
onStartTyping(() => {
  titleRef.value?.$el?.focus()
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

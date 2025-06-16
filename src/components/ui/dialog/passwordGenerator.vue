<template>
  <UiDialogConfirm
    :confirm-label="t('apply')"
    :title="t('title')"
    @abort="open = false"
    @click="open = true"
    class="btn btn-square btn-accent btn-outline"
    v-model:open="open"
  >
    <template #trigger>
      <Icon name="mdi:dice" />
    </template>

    <form class="flex flex-col gap-4">
      <UiInputPassword
        v-model="newPassword"
        prepend-icon="mdi:key-outline"
        with-copy-button
      >
        <template #append>
          <UiButton class="btn-square btn-accent btn-outline">
            <Icon name="mdi:refresh" />
          </UiButton>
        </template>
      </UiInputPassword>
    </form>
  </UiDialogConfirm>
</template>

<script setup lang="ts">
const open = defineModel<boolean>()

const { t } = useI18n()

const { password } = defineProps<{
  autofocus?: boolean
  checkInput?: boolean
  label?: string
  placeholder?: string
  withCopyButton?: boolean
  password: string | null
}>()

const newPassword = computed(() => password)
</script>

<i18n lang="yaml">
de:
  title: Passwortgenerator
  apply: Übernehmen

en:
  title: Passwordgenerator
  apply: Apply
</i18n>

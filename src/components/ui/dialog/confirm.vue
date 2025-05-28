<template>
  <UiDialog v-model:open="open" @close="onAbort">
    <template #trigger>
      <slot name="trigger" />
    </template>

    <template #title>
      <slot name="title" />
    </template>

    <slot />

    <template #buttons>
      <slot name="buttons">
        <UiButton class="btn-error btn-outline" @click="onAbort">
          <Icon name="mdi:close" /> {{ abortLabel ?? t("abort") }}
        </UiButton>
        <UiButton class="btn-primary " @click="onConfirm">
          <Icon name="mdi:check" /> {{ confirmLabel ?? t("confirm") }}
        </UiButton>
      </slot>

    </template>
  </UiDialog>
</template>

<script setup lang="ts">
defineProps<{ confirmLabel?: string, abortLabel?: string }>()
const open = defineModel<boolean>("open", { default: false })
const { t } = useI18n()
const emit = defineEmits(["confirm", "abort"])

const onAbort = () => {
  open.value = false
  emit("abort")
}

const onConfirm = () => {
  open.value = false
  emit("confirm")
}
</script>

<i18n lang="yaml">
  de:
    abort: Abbrechen
    confirm: Bestätigen

  en:
    abort: Abort
    confirm: Confirm
</i18n>
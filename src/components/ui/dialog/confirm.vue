<template>
  <UModal
    v-model:open="open"
    :title
    :description
  >
    <template
      v-for="(_, name) in $slots"
      :key="name"
      #[name]="slotData"
    >
      <slot
        :name="name"
        v-bind="slotData"
      />
    </template>

    <template #footer>
      <div class="flex flex-col sm:flex-row gap-4 justify-end w-full">
        <UiButton
          :icon="abortIcon || 'mdi:close'"
          :label="abortLabel || t('abort')"
          block
          color="error"
          variant="outline"
          @click="open = false"
        />
        <UiButton
          :icon="confirmIcon || 'mdi:check'"
          :label="confirmLabel || t('confirm')"
          block
          color="primary"
          variant="solid"
          @click="$emit('confirm')"
        />
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
defineProps<{
  abortIcon?: string
  abortLabel?: string
  confirmIcon?: string
  confirmLabel?: string
  description?: string
  title?: string
}>()

const open = defineModel<boolean>('open', { default: false })

const { t } = useI18n()
defineEmits(['confirm'])
</script>

<i18n lang="yaml">
de:
  abort: Abbrechen
  confirm: Bestätigen

en:
  abort: Abort
  confirm: Confirm
</i18n>

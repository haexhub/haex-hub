<template>
  <HaexPassCard
    :title
    @close="onClose"
  >
    <div class="flex flex-col gap-4 w-full p-4">
      <slot />

      <UiInput
        v-show="!read_only"
        v-model.trim="passwordGroup.name"
        :label="t('group.name')"
        :placeholder="t('group.name')"
        :with-copy-button="read_only"
        :read_only
        autofocus
      />

      <UiInput
        v-show="!read_only || passwordGroup.description?.length"
        v-model.trim="passwordGroup.description"
        :read_only
        :label="t('group.description')"
        :placeholder="t('group.description')"
        :with-copy-button="read_only"
      />

      <UiSelectColor
        v-model="passwordGroup.color"
        :read_only
        :label="t('group.color')"
        :placeholder="t('group.color')"
      />

      <UiSelectIcon
        v-model="passwordGroup.icon"
        :read_only
        :label="t('group.icon')"
        :placeholder="t('group.icon')"
      />
    </div>

    <slot name="footer" />
  </HaexPassCard>
</template>

<script setup lang="ts">
import type { RouteLocationNormalizedLoadedGeneric } from 'vue-router'
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

const { t } = useI18n()
const showConfirmation = ref(false)
const passwordGroup = defineModel<SelectHaexPasswordsGroups>({ required: true })
const read_only = defineModel<boolean>('read_only')
const props = defineProps<{
  originally: SelectHaexPasswordsGroups
  title: string
}>()

defineEmits<{
  submit: [to?: RouteLocationNormalizedLoadedGeneric]
  close: [void]
  back: [void]
  reject: [to?: RouteLocationNormalizedLoadedGeneric]
}>()

const hasChanges = computed(() => {
  console.log('group has changes', props.originally, passwordGroup.value)
  if (!props.originally) {
    if (
      passwordGroup.value.color?.length ||
      passwordGroup.value.description?.length ||
      passwordGroup.value.icon?.length ||
      passwordGroup.value.name?.length
    ) {
      return true
    } else {
      return false
    }
  }
  return (
    JSON.stringify(props.originally) !== JSON.stringify(passwordGroup.value)
  )
})

const onClose = () => {
  /* if (props.originally) passwordGroup.value = { ...props.originally };
  emit('close'); */
  console.log('close group card')
}
</script>

<i18n lang="yaml">
de:
  group:
    name: Name
    description: Beschreibung
    icon: Icon
    color: Farbe

en:
  group:
    name: Name
    description: Description
    icon: Icon
    color: Color
</i18n>

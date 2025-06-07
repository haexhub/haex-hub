<template>
  <VaultCardEdit
    v-if="vaultGroup"
    v-model:read_only="read_only"
    :color="vaultGroup.color ?? 'text-base-content'"
    :has-changes="hasChanges"
    :icon="vaultGroup.icon ?? 'mdi:folder-outline'"
    :title="vaultGroup.name ?? ''"
    @back="$emit('back')"
    @close="$emit('close')"
    @reject="(to) => $emit('reject', to)"
    @submit="(to) => $emit('submit', to)"
  >
    <div class="flex flex-col gap-4 w-full p-4">
      <UiInput
        v-show="!read_only"
        v-model.trim="vaultGroup.name"
        :label="t('vaultGroup.name')"
        :placeholder="t('vaultGroup.name')"
        :with-copy-button="read_only"
        :read_only
        autofocus
      />

      <UiInput
        v-show="!read_only || vaultGroup.description?.length"
        v-model.trim="vaultGroup.description"
        :read_only
        :label="t('vaultGroup.description')"
        :placeholder="t('vaultGroup.description')"
        :with-copy-button="read_only"
      />

      <UiSelectColor
        v-model="vaultGroup.color"
        :read_only
        :label="t('vaultGroup.color')"
        :placeholder="t('vaultGroup.color')"
      />

      <UiSelectIcon
        v-model="vaultGroup.icon"
        :read_only
        :label="t('vaultGroup.icon')"
        :placeholder="t('vaultGroup.icon')"
      />
    </div>
  </VaultCardEdit>
</template>

<script setup lang="ts">
import type { RouteLocationNormalizedLoadedGeneric } from 'vue-router'
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

const { t } = useI18n()
const showConfirmation = ref(false)
const vaultGroup = defineModel<SelectHaexPasswordsGroups>({ required: true })
const read_only = defineModel<boolean>('read_only')
const props = defineProps({
  originally: Object as PropType<SelectHaexPasswordsGroups>,
})

defineEmits<{
  submit: [to?: RouteLocationNormalizedLoadedGeneric]
  close: [void]
  back: [void]
  reject: [to?: RouteLocationNormalizedLoadedGeneric]
}>()

const hasChanges = computed(() => {
  console.log('group has changes', props.originally, vaultGroup.value)
  if (!props.originally) {
    if (
      vaultGroup.value.color?.length ||
      vaultGroup.value.description?.length ||
      vaultGroup.value.icon?.length ||
      vaultGroup.value.name?.length
    ) {
      return true
    } else {
      return false
    }
  }
  return JSON.stringify(props.originally) !== JSON.stringify(vaultGroup.value)
})

/* const onClose = () => {
  if (props.originally) vaultGroup.value = { ...props.originally };
  emit('close');
}; */
</script>

<i18n lang="json">
{
  "de": {
    "vaultGroup": {
      "name": "Name",
      "description": "Beschreibung",
      "icon": "Icon",
      "color": "Farbe"
    }
  },
  "en": {
    "vaultGroup": {
      "name": "Name",
      "description": "Description",
      "icon": "Icon",
      "color": "Color"
    }
  }
}
</i18n>

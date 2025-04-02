<template>
  <VaultCardEdit
    v-if="vaultGroup"
    :color="vaultGroup.color ?? 'text-base-content'"
    :has-changes="hasChanges"
    :icon="vaultGroup.icon ?? 'mdi:folder-outline'"
    :title="vaultGroup.name ?? ''"
    @back="$emit('back')"
    @close="$emit('close')"
    @reject="(to) => $emit('reject', to)"
    @submit="(to) => $emit('submit', to)"
    v-model:read_only="read_only"
  >
    <div class="flex flex-col gap-4 w-full p-4">
      <UiInput
        v-show="!read_only"
        :label="t('vaultGroup.name')"
        :placeholder="t('vaultGroup.name')"
        :rules="vaultGroupSchema.name"
        :with-copy-button="read_only"
        :read_only
        autofocus
        v-model.trim="vaultGroup.name"
      />

      <UiInput
        v-show="!read_only || vaultGroup.description?.length"
        :read_only
        :label="t('vaultGroup.description')"
        :placeholder="t('vaultGroup.description')"
        :rules="vaultGroupSchema.description"
        :with-copy-button="read_only"
        v-model.trim="vaultGroup.description"
      />

      <UiColorPicker
        :read_only
        :label="t('vaultGroup.color')"
        :placeholder="t('vaultGroup.color')"
        v-model="vaultGroup.color"
      />

      <UiIconPicker
        :read_only
        :label="t('vaultGroup.icon')"
        :placeholder="t('vaultGroup.icon')"
        v-model="vaultGroup.icon"
      />
    </div>
  </VaultCardEdit>
</template>

<script setup lang="ts">
import type { RouteLocationNormalizedLoadedGeneric } from 'vue-router';
import {
  vaultGroupSchema,
  type SelectVaultGroup,
} from '~/database/schemas/vault';

const { t } = useI18n();
const showConfirmation = ref(false);
const vaultGroup = defineModel<SelectVaultGroup>({ required: true });
const read_only = defineModel<boolean>('read_only');
const props = defineProps({
  originally: Object as PropType<SelectVaultGroup>,
});

defineEmits<{
  submit: [to?: RouteLocationNormalizedLoadedGeneric];
  close: [void];
  back: [void];
  reject: [to?: RouteLocationNormalizedLoadedGeneric];
}>();

const hasChanges = computed(() => {
  console.log('group has changes', props.originally, vaultGroup.value);
  if (!props.originally) {
    if (
      vaultGroup.value.color?.length ||
      vaultGroup.value.description?.length ||
      vaultGroup.value.icon?.length ||
      vaultGroup.value.name?.length
    ) {
      return true;
    } else {
      return false;
    }
  }
  return JSON.stringify(props.originally) !== JSON.stringify(vaultGroup.value);
});

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

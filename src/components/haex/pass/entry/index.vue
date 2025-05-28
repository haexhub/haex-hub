<template>
  <VaultCardEdit
v-if="vaultEntry.details" v-model:read_only="read_only" :color="currentGroup?.color || 'text-base-content'"
    :has-changes="hasChanges" :icon="vaultEntry.details?.icon || icon || 'mdi:key-outline'"
    :title="vaultEntry.details?.title ?? ''" @back="$emit('back')" @close="$emit('close')"
    @reject="(to) => $emit('reject', to)" @submit="(to) => $emit('submit', to)">
    <div class="h-full relative overflow-hidden">
      <nav
aria-label="Tabs Vault Entry" aria-orientation="horizontal"
        class="tabs tabs-bordered w-full transition-all duration-700 sticky top-0 z-10 bg-base-200" role="tablist">
        <button
:id="id.details" aria-controls="vaultDetailsId" aria-selected="true"
          class="tab active-tab:tab-active active w-full" data-tab="#vaultDetailsId" role="tab" type="button">
          <Icon name="material-symbols:key-outline" class="me-2" />
          <span class="hidden sm:block">
            {{ t('tab.details') }}
          </span>
        </button>
        <button
:id="id.keyValue" aria-controls="tabs-basic-2" aria-selected="false"
          class="tab active-tab:tab-active w-full" data-tab="#tabs-basic-2" role="tab" type="button">
          <Icon name="fluent:group-list-20-filled" class="me-2" />
          <span class="hidden sm:block">
            {{ t('tab.keyValue') }}
          </span>
        </button>
        <button
:id="id.history" aria-controls="tabs-basic-3" aria-selected="false"
          class="tab active-tab:tab-active w-full" data-tab="#tabs-basic-3" role="tab" type="button">
          <Icon name="material-symbols:history" class="me-2" />
          <span class="hidden sm:block">
            {{ t('tab.history') }}
          </span>
        </button>
      </nav>

      <div class="h-full pb-8">
        <div id="vaultDetailsId" role="tabpanel" :aria-labelledby="id.details" class="h-full">
          <VaultEntryDetails v-if="vaultEntry.details" v-model="vaultEntry.details" :with-copy-button :read_only/>
        </div>

        <div id="tabs-basic-2" class="hidden" role="tabpanel" :aria-labelledby="id.keyValue">
          {{ originally }}
        </div>

        <div id="tabs-basic-3" class="hidden h-full" role="tabpanel" :aria-labelledby="id.history">
          <VaultEntryHistory v-if="vaultEntry.history" :history="vaultEntry.history" />
        </div>
      </div>
    </div>
  </VaultCardEdit>
  <!-- <VaultModalSaveChanges
    v-model="showConfirmation"
    @reject="onReject"
    @submit="onSubmit"
  /> -->
</template>

<script setup lang="ts">

import type { RouteLocationNormalizedLoadedGeneric } from 'vue-router';

const { t } = useI18n();

const vaultEntry = defineModel<IVaultEntryComplete>({ required: true });


const { currentGroup } = storeToRefs(useVaultGroupStore());

/* watch(
  () => vaultEntry.value.details,
  () => {
    header.value.text = vaultEntry.value.details?.title;
    header.value.icon =
      vaultEntry.value.details?.icon || currentGroup.value?.icon;
  },
  { immediate: true }
); */

const id = reactive({
  details: useId(),
  keyValue: useId(),
  history: useId(),
  content: {},
});

const read_only = defineModel<boolean>('read_only', { default: false });

const props = defineProps({
  icon: String,
  originally: Object as PropType<IVaultEntryComplete>,
  title: String,
  withCopyButton: Boolean,
});

const emit = defineEmits<{
  submit: [to?: RouteLocationNormalizedLoadedGeneric];
  close: [void];
  back: [void];
  reject: [to?: RouteLocationNormalizedLoadedGeneric];
}>();

const showConfirmation = ref(false);

const hasChanges = computed(() => {
  if (!props.originally?.details) {
    if (
      vaultEntry.value.details?.note?.length ||
      vaultEntry.value.details?.password?.length ||
      vaultEntry.value.details?.tags?.length ||
      vaultEntry.value.details?.title?.length ||
      vaultEntry.value.details?.url?.length ||
      vaultEntry.value.details?.urlAliases?.length ||
      vaultEntry.value.details?.username?.length
    ) {
      console.log('has changes', props.originally, vaultEntry.value);
      return true;
    } else {
      return false;
    }
  }
  return (
    JSON.stringify(props.originally.details) !==
    JSON.stringify(vaultEntry.value.details)
  );
});

const to = ref<RouteLocationNormalizedLoadedGeneric>();

const isSaved = ref(false);
const isRejected = ref(false);

const onSubmit = () => {
  console.log('entry onSubmit');
  showConfirmation.value = false;
  isSaved.value = true;
  emit('submit', to.value);
};

const onReject = () => {
  console.log('entry onReject');
  showConfirmation.value = false;
  isRejected.value = true;
  emit('reject', to.value);
};

const onBack = () => {
  console.log('entry onBack', read_only.value);
  if (hasChanges.value) {
    showConfirmation.value = true;
  } else {
    emit('back');
  }
};

/* onBeforeRouteLeave((_to, _from, next) => {
  console.log('check before leave', _to, _from);
  to.value = _to;
  if (isSaved.value || isRejected.value) {
    isSaved.value = false;
    isRejected.value = false;
    next();
  } else if (hasChanges.value) {
    showConfirmation.value = true;
  } else {
    next();
  }
}); */
</script>

<i18n lang="json">{
  "de": {
    "create": "Anlegen",
    "abort": "Abbrechen",
    "tab": {
      "details": "Details",
      "keyValue": "Extra",
      "history": "Verlauf"
    }
  },
  "en": {
    "create": "Create",
    "abort": "Abort",
    "tab": {
      "details": "Details",
      "keyValue": "Extra",
      "history": "History"
    }
  }
}</i18n>
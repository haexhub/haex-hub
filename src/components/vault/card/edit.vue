<template>
  <VaultCard
    @close="onClose"
    @submit="onSubmit"
  >
    <template #header>
      <div class="flex flex-wrap items-center justify-between w-full px-2 py-3">
        <div class="w-full flex gap-2 justify-between items-center">
          <div class="flex items-center gap-2">
            <button
              class="btn btn-square btn-primary btn-outline"
              @click="onBack"
            >
              <Icon
                name="mdi:chevron-left"
                size="32"
              />
            </button>

            <button
              class="btn btn-square btn-error btn-outline"
              @click="onBack"
            >
              <Icon
                name="mdi:trash-can-outline"
                size="28"
              />
            </button>
          </div>
          <slot name="buttons">
            <div
              v-if="read_only"
              class="h-full"
            >
              <button
                class="btn btn-square btn-primary btn-outline"
                @click="read_only = false"
              >
                <Icon
                  name="mdi:pencil-outline"
                  size="24"
                />
              </button>
            </div>
            <div
              v-else
              class="gap-2 h-full hidden md:flex"
            >
              <button
                class="btn btn-square btn-error btn-outline"
                @click="onClose"
              >
                <Icon name="mdi:close" />
                <span class="hidden"> {{ t('abort') }} </span>
              </button>
              <button
                class="btn btn-square btn-success btn-outline"
                @click="onSubmit"
              >
                <Icon name="mdi:check" />
                <span class="hidden"> {{ t('create') }} </span>
              </button>
            </div>
          </slot>
        </div>

        <div
          class="flex flex-col items-center w-full min-h-14 gap-2 py-1"
          :class="{ '-ml-6': !show }"
          :style="{ color }"
        >
          <Icon
            v-if="icon"
            :name="icon"
            size="28"
          />

          <h5
            v-show="read_only"
            class="overflow-hidden whitespace-nowrap"
          >
            {{ title }}
          </h5>
        </div>
      </div>
    </template>

    <div class="h-full">
      <slot />
      <div
        v-show="!read_only"
        class="fixed bottom-2 left-0 w-full flex items-center justify-between px-4 md:hidden"
      >
        <div
          class="transition-all duration-500"
          :class="{ 'pl-96': show }"
        >
          <button
            class="btn btn-square btn-error btn-outline"
            @click="onClose"
          >
            <Icon name="mdi:close" />
            <span class="hidden"> {{ t('abort') }} </span>
          </button>
        </div>
        <div>
          <button
            class="btn btn-square btn-success"
            @click="onSubmit"
          >
            <Icon name="mdi:check" />
            <span class="hidden"> {{ t('create') }} </span>
          </button>
        </div>
        <div></div>
      </div>
      <!-- <UiButtonAction
        class=""
        icon="mdi:content-save-outline"
        ><Icon name="mdi:content-save-outline" />
      </UiButtonAction> -->
    </div>
  </VaultCard>
  <VaultModalSaveChanges
    v-model="showConfirmation"
    @reject="onReject"
    @submit="onSubmit"
  />
</template>

<script setup lang="ts">
import type { RouteLocationNormalizedLoadedGeneric } from 'vue-router';

const { t } = useI18n();

const { show } = storeToRefs(useSidebarStore());

const read_only = defineModel<boolean>('read_only', { default: false });

const props = defineProps({
  color: String,
  hasChanges: Boolean,
  icon: String,
  title: String,
});

const emit = defineEmits<{
  back: [void];
  close: [void];
  reject: [to?: RouteLocationNormalizedLoadedGeneric];
  submit: [to?: RouteLocationNormalizedLoadedGeneric];
}>();

const showConfirmation = ref(false);

const to = ref<RouteLocationNormalizedLoadedGeneric>();

const isApprovedForLeave = ref(false);

const wantToGoBack = ref(false);

const onSubmit = () => {
  showConfirmation.value = false;
  isApprovedForLeave.value = true;
  if (wantToGoBack.value) {
    wantToGoBack.value = false;
    read_only.value = true;
    emit('submit');
  } else {
    emit('submit', to.value);
  }
};

const onReject = () => {
  showConfirmation.value = false;
  isApprovedForLeave.value = true;
  read_only.value = true;

  if (wantToGoBack.value) {
    wantToGoBack.value = false;
    emit('back');
  } else emit('reject', to.value);
};

const onBack = () => {
  if (props.hasChanges) {
    wantToGoBack.value = true;
    showConfirmation.value = true;
  } else {
    emit('back');
  }
};

const onClose = () => {
  if (props.hasChanges) {
    showConfirmation.value = true;
  } else {
    emit('close'); //read_only.value = true;
  }
};

const onDelete = () => {};
onBeforeRouteLeave((_to, _from, next) => {
  //console.log('check before leave', _to, _from);
  to.value = _to;
  if (isApprovedForLeave.value) {
    isApprovedForLeave.value = false;
    next();
  } else if (props.hasChanges) {
    showConfirmation.value = true;
  } else {
    next();
  }
});
</script>

<i18n lang="json">
{
  "de": {
    "create": "Anlegen",
    "abort": "Abbrechen",
    "entry": {
      "title": "Titel",
      "username": "Nutzername",
      "password": "Passwort",
      "url": "Url"
    },
    "tab": {
      "details": "Details",
      "keyValue": "Extra",
      "history": "Verlauf"
    }
  },
  "en": {
    "create": "Create",
    "abort": "Abort",
    "entry": {
      "title": "Title",
      "username": "Username",
      "password": "Password",
      "url": "Url"
    },
    "tab": {
      "details": "Details",
      "keyValue": "Extra",
      "history": "History"
    }
  }
}
</i18n>

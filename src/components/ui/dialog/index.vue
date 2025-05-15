<template>
  <slot name="trigger" :id> </slot>

  <div
    :id
    class="overlay modal overlay-open:opacity-100 hidden modal-middle [--tab-accessibility-limited:false] overflow-scroll p-0 sm:p-4"
    role="dialog"
    ref="modalRef"
  >
    <div
      class="overlay-animation-target overlay-open:mt-4 overlay-open:duration-500 mt-12 transition-all ease-out modal-dialog overlay-open:opacity-100"
    >
      <div class="modal-content">
        <div class="modal-header">
          <slot name="title">
            <h3 v-if="title" class="modal-title text-base sm:text-lg">
              {{ title }}
            </h3>
          </slot>

          <button
            type="button"
            class="btn btn-text btn-circle btn-sm absolute end-3 top-3"
            :aria-label="t('close')"
            @click="open = false"
            tabindex="1"
          >
            <Icon name="mdi:close" size="18" />
          </button>
        </div>
        <div class="modal-body text-sm sm:text-base py-1">
          <slot />
        </div>
        <div class="modal-footer flex-wrap">
          <slot name="buttons" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { HSOverlay } from "flyonui/flyonui";

export interface IDom {
  class?: String;
  text: String;
}

const id = useId();

defineProps({
  title: {
    type: String,
    default: "",
  },
});

const open = defineModel<boolean>("open", { default: false });

const { t } = useI18n();
const modalRef = useTemplateRef("modalRef");
const modal = ref<HSOverlay>();

watch(open, async () => {
  console.log("open modal", open.value);
  if (open.value) {
    await modal.value?.open();
  } else {
    await modal.value?.close(true);
    console.log("close dialog");
  }
});

onMounted(() => {
  if (!modalRef.value) return;
  modal.value = new HSOverlay(modalRef.value, { isClosePrev: true });

  modal.value.on("close", () => {
    console.log("close it from event", open.value);
    open.value = false;
  });
});
</script>

<i18n lang="json">
{
  "de": {
    "close": "Schließen"
  },
  "en": {
    "close": "Close"
  }
}
</i18n>

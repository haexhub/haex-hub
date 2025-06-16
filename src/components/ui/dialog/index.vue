<template>
  <button
    v-if="$slots.trigger || label"
    v-bind="$attrs"
    type="button"
    aria-haspopup="dialog"
    aria-expanded="false"
    :aria-label="label"
    @click="$emit('open')"
  >
    <slot name="trigger">
      {{ label }}
    </slot>
  </button>

  <Teleport to="body">
    <div
      :id
      ref="modalRef"
      class="overlay modal overlay-open:opacity-100 overlay-open:duration-300 hidden modal-middle p-0 xs:p-2 --prevent-on-load-init pointer-events-auto max-w-none"
      role="dialog"
      tabindex="-1"
    >
      <div
        class="overlay-animation-target overlay-open:duration-300 overlay-open:opacity-100 transition-all ease-out modal-dialog"
      >
        <div class="modal-content justify-between">
          <div class="modal-header">
            <div
              v-if="title || $slots.title"
              class="modal-title py-4 break-all"
            >
              <slot name="title">
                {{ title }}
              </slot>
            </div>

            <button
              type="button"
              class="btn btn-text btn-circle btn-sm absolute end-3 top-3"
              :aria-label="t('close')"
              tabindex="1"
              @click="open = false"
            >
              <Icon
                name="mdi:close"
                size="18"
              />
            </button>
          </div>

          <div class="modal-body text-sm sm:text-base grow">
            <slot />
          </div>

          <div class="modal-footer flex-col sm:flex-row">
            <slot name="buttons" />
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import type { HSOverlay } from 'flyonui/flyonui'
const { currentTheme } = storeToRefs(useUiStore())

defineProps<{ title?: string; label?: string }>()

const emit = defineEmits(['open', 'close'])

const id = useId()

const open = defineModel<boolean>('open', { default: false })

const { t } = useI18n()

const modalRef = useTemplateRef('modalRef')

defineExpose({ modalRef })

const modal = ref<HSOverlay>()

watch(open, async () => {
  if (!modal.value) return

  if (open.value) {
    await modal.value.open()
  } else {
    await modal.value.close(true)
    emit('close')
  }
})

onMounted(async () => {
  if (!modalRef.value) return

  modal.value = new window.HSOverlay(modalRef.value)

  modal.value.isLayoutAffect = true
  modal.value.on('close', () => {
    open.value = false
  })
})
</script>

<i18n lang="yaml">
de:
  close: Schließen

en:
  close: Close
</i18n>

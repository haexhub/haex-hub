<template>
  <div>
    <fieldset
      class="join w-full"
      :class="{ 'pt-1.5': label }"
      v-bind="$attrs"
    >
      <slot name="prepend" />

      <div class="input join-item">
        <Icon
          v-if="prependIcon"
          :name="prependIcon"
          class="my-auto shrink-0"
        />

        <div class="input-floating grow">
          <input
            :autofocus
            :id
            :name="name ?? id"
            :placeholder="placeholder || label"
            :readonly="read_only"
            :type
            class="ps-2"
            ref="inputRef"
            v-model="input"
            @keyup="(e) => $emit('keyup', e)"
          />
          <label
            :for="id"
            class="input-floating-label"
          >
            {{ label }}
          </label>
        </div>

        <Icon
          v-if="appendIcon"
          :name="appendIcon"
          class="my-auto shrink-0"
        />
      </div>

      <slot
        name="append"
        class="h-auto"
      />

      <UiButton
        v-if="withCopyButton"
        :tooltip="t('copy')"
        class="btn-outline btn-accent btn-square"
        @click="copy(`${input}`)"
      >
        <Icon :name="copied ? 'mdi:check' : 'mdi:content-copy'" />
      </UiButton>
    </fieldset>

    <span
      v-show="errors"
      class="flex flex-col px-2 pt-0.5"
    >
      <span
        v-for="error in errors"
        class="label-text-alt text-error"
      >
        {{ error }}
      </span>
    </span>
  </div>
</template>

<script setup lang="ts">
import type { ZodSchema } from 'zod'

const input = defineModel<string | number | undefined | null>({
  required: true,
})

const inputRef = useTemplateRef('inputRef')
defineExpose({ inputRef })

const emit = defineEmits<{
  error: [string[]]
  keyup: [KeyboardEvent]
}>()

const props = defineProps({
  placeholder: {
    type: String,
    default: '',
  },
  type: {
    type: String as PropType<
      | 'button'
      | 'checkbox'
      | 'color'
      | 'date'
      | 'datetime-local'
      | 'email'
      | 'file'
      | 'hidden'
      | 'image'
      | 'month'
      | 'number'
      | 'password'
      | 'radio'
      | 'range'
      | 'reset'
      | 'search'
      | 'submit'
      | 'tel'
      | 'text'
      | 'time'
      | 'url'
      | 'week'
    >,
    default: 'text',
  },
  label: String,
  name: String,
  prependIcon: {
    type: String,
    default: '',
  },
  prependLabel: String,
  appendIcon: {
    type: String,
    default: '',
  },
  appendLabel: String,
  rules: Object as PropType<ZodSchema>,
  checkInput: Boolean,
  withCopyButton: Boolean,
  autofocus: Boolean,
  read_only: Boolean,
})

onMounted(() => {
  if (props.autofocus && inputRef.value) inputRef.value.focus()
})

const errors = defineModel<string[] | undefined>('errors')

const id = useId()

watch(input, () => checkInput())

watch(
  () => props.checkInput,
  () => {
    checkInput()
  },
)

const checkInput = () => {
  if (props.rules) {
    const result = props.rules.safeParse(input.value)
    //console.log('check result', result.error, props.rules);
    if (!result.success) {
      errors.value = result.error.errors.map((error) => error.message)
      emit('error', errors.value)
    } else {
      errors.value = []
    }
  }
}

const { copy, copied } = useClipboard()

const { t } = useI18n()
</script>

<i18n lang="yaml">
de:
  copy: Kopieren

en:
  copy: Copy
</i18n>

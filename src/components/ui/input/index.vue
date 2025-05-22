<template>
  <div>
    <fieldset class="join w-full pt-0.5">
      <slot name="prepend" />

      <div class="input join-item">
        <Icon v-if="prependIcon" :name="prependIcon" class="my-auto shrink-0" />

        <div class="input-floating grow">
          <input :id :name="name ?? id" :placeholder="placeholder || label" :type :autofocus class="ps-3"
            v-bind="$attrs" v-model="input" ref="inputRef" :readonly="read_only" />
          <label class="input-floating-label" :for="id">{{ label }}</label>
        </div>

        <Icon v-if="appendIcon" :name="appendIcon" class="my-auto shrink-0" />
      </div>

      <slot name="append" class="h-auto" />

      <UiButton v-if="withCopyButton" class="btn-outline btn-accent btn-square join-item h-auto"
        @click="copy(`${input}`)">
        <Icon :name="copied ? 'mdi:check' : 'mdi:content-copy'" />
      </UiButton>
    </fieldset>

    <span class="flex flex-col px-2 pt-0.5" v-show="errors">
      <span v-for="error in errors" class="label-text-alt text-error">
        {{ error }}
      </span>
    </span>
  </div>
</template>

<script setup lang="ts">
import { type ZodSchema } from "zod";

const inputRef = useTemplateRef("inputRef");
defineExpose({ inputRef });

defineOptions({
  inheritAttrs: false,
});

const props = defineProps({
  placeholder: {
    type: String,
    default: "",
  },
  type: {
    type: String as PropType<
      | "button"
      | "checkbox"
      | "color"
      | "date"
      | "datetime-local"
      | "email"
      | "file"
      | "hidden"
      | "image"
      | "month"
      | "number"
      | "password"
      | "radio"
      | "range"
      | "reset"
      | "search"
      | "submit"
      | "tel"
      | "text"
      | "time"
      | "url"
      | "week"
    >,
    default: "text",
  },
  label: String,
  name: String,
  prependIcon: {
    type: String,
    default: "",
  },
  prependLabel: String,
  appendIcon: {
    type: String,
    default: "",
  },
  appendLabel: String,
  rules: Object as PropType<ZodSchema>,
  checkInput: Boolean,
  withCopyButton: Boolean,
  autofocus: Boolean,
  read_only: Boolean,
});

const input = defineModel<string | number | undefined | null>({
  default: "",
  required: true,
});

onMounted(() => {
  if (props.autofocus && inputRef.value) inputRef.value.focus();
});

const errors = defineModel<string[] | undefined>("errors");

const id = useId();

watch(input, () => checkInput());

watch(
  () => props.checkInput,
  () => {
    checkInput();
  }
);

const emit = defineEmits(["error"]);

const checkInput = () => {
  if (props.rules) {
    const result = props.rules.safeParse(input.value);
    //console.log('check result', result.error, props.rules);
    if (!result.success) {
      errors.value = result.error.errors.map((error) => error.message);
      emit("error", errors.value);
    } else {
      errors.value = [];
    }
  }
};

const { copy, copied } = useClipboard();
</script>
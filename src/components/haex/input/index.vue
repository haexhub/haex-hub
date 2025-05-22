<template>
  <div>
    <fieldset class="join w-full">
      <slot name="prepend" />


      <!-- <div class="">
      -->
      <HaexButton v-if="withCopyButton" class="btn-outline btn-accent btn-square join-item h-auto"
        @click="copy(`${input}`)">
        <Icon :name="copied ? 'mdi:check' : 'mdi:content-copy'" />
      </Haexbutton>

      <!-- <div class="">
          <input :id :name="name ?? id" :placeholder="placeholder || label" :type :autofocus class="" v-bind="$attrs"
            v-model="input" ref="inputRef" :readonly="read_only" />
          <label class="floating-label" :for="id">{{ label }}</label>
        </div> -->
      <label class="floating-label input join-item">
        <Icon v-if="iconPrepend" :name="iconPrepend" class="my-auto size-6" />
        <span>Your Email</span>
        <input type="text" placeholder="mail@site.com" class=" join-item " />
        <Icon v-if="iconAppend" :name="iconAppend" class="my-auto shrink-0" />
      </label>

      <!-- <Icon v-if="iconAppend" :name="iconAppend" class="my-auto shrink-0" />
      </div> -->

      <slot name="append" class="h-auto" />

      <HaexButton v-if="withCopyButton" class="btn-outline btn-accent btn-square join-item h-auto"
        @click="copy(`${input}`)">
        <Icon :name="copied ? 'mdi:check' : 'mdi:content-copy'" />
      </Haexbutton>
    </fieldset>

    <span class="flex flex-col px-2 pt-0.5" v-show="errors">
      <span v-for="error in errors" class="label-text-alt text-error">
        {{ error }}
      </span>
    </span>
  </div>
  <!-- <div class="relative w-full max-w-sm items-center">
    <span class="absolute start-0 inset-y-0 flex items-center justify-center px-2">
      <Icon v-if="iconPrepend" :name="iconPrepend" class="size-6" />
      <button>aa</button>
    </span>

    <Input id="search" type="text" placeholder="Search..." :class="{ 'pl-10': iconPrepend, 'pr-10': iconAppend }" />

    <span class="absolute end-0 inset-y-0 flex items-center justify-center px-2">
      <Icon v-if="iconAppend" :name="iconAppend" class="size-6" />
    </span>
  </div> -->
</template>

<script setup lang="ts">
import { HaexButton } from '#components';
import type { ZodSchema } from 'zod';

const id = useId()

const props = defineProps<{ iconAppend?: string, iconPrepend?: string, placeholder?: string, type?: string, label?: string, withCopyButton?: boolean, rules?: ZodSchema, read_only?: boolean, autofocus?: boolean, checkInput?: boolean, name?: string }>()

const inputRef = useTemplateRef('inputRef')

const input = defineModel<string | number | undefined | null>({
  default: '',
  required: true,
})

onMounted(() => {
  if (props.autofocus && inputRef.value) inputRef.value.focus()
})

watch(input, () => checkInput())

watch(
  () => props.checkInput,
  () => {
    checkInput()
  }
)

const emit = defineEmits(['error'])

const errors = defineModel<string[] | undefined>('errors')
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
</script>
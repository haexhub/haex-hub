<template>
  <div>
    <VaultCard
      :title="t('title')"
      icon="mdi:folder-plus-outline"
      @close="onClose"
    >
      <div
        class="flex flex-col gap-4 w-full p-4"
        @keyup.enter="onCreate"
      >
        <UiInput
          :check-input="check"
          :label="t('name.label')"
          :placeholder="t('name.label')"
          autofocus
          v-model:errors="errors.name"
          v-model="vaultGroup.name"
        />

        <UiInput
          v-model="vaultGroup.description"
          :check-input="check"
          :label="t('description.label')"
          :placeholder="t('description.label')"
        />

        <UiSelectColor v-model="vaultGroup.color" />

        {{ vaultGroup.icon }}
        <UiSelectIcon v-model="vaultGroup.icon" />

        <div class="flex flex-wrap justify-end gap-4">
          <button
            class="btn btn-error btn-outline flex-1 flex-nowrap"
            @click="onClose"
            type="button"
          >
            {{ t('abort') }}
            <Icon name="mdi:close" />
          </button>
          <button
            class="btn btn-primary flex-1 flex-nowrap"
            type="button"
            @click="onCreate"
          >
            {{ t('create') }}
            <Icon name="mdi:check" />
          </button>
        </div>
      </div>
    </VaultCard>
  </div>
</template>

<script setup lang="ts">
import type { InsertHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

definePageMeta({
  name: 'passwordGroupCreate',
})

const { t } = useI18n()

const check = ref(false)

const { currentGroupId } = storeToRefs(usePasswordGroupStore())
const vaultGroup = ref<InsertHaexPasswordsGroups>({
  name: '',
  description: '',
  id: '',
  color: null,
  icon: null,
  order: null,
  parentId: currentGroupId.value,
})

const errors = ref({
  name: [],
  description: [],
})

const onClose = () => {
  useRouter().back()
}

const onCreate = async () => {
  try {
    check.value = true

    if (errors.value.name.length || errors.value.description.length) return

    const { addGroupAsync } = usePasswordGroupStore()

    const newGroup = await addGroupAsync(vaultGroup.value)

    console.log('newGroup', newGroup)
    if (!newGroup.id) {
      return
    }
    //console.log('created group with id', newGroup?.id)

    //currentGroupId.value = newGroup?.id
    await navigateTo(
      useLocalePath()({
        name: 'passwordGroupItems',
        params: {
          groupId: newGroup.id,
        },
        query: {
          ...useRoute().query,
        },
      }),
    )
  } catch (error) {
    console.log(error)
  }
}
</script>

<i18n lang="json">
{
  "de": {
    "title": "Neue Gruppe anlegen",
    "abort": "Abbrechen",
    "create": "Anlegen",
    "name": {
      "label": "Name"
    },
    "description": {
      "label": "Beschreibung"
    }
  },

  "en": {
    "title": "Create new Group",
    "abort": "Abort",
    "create": "Create",
    "name": {
      "label": "Name"
    },
    "description": {
      "label": "Description"
    }
  }
}
</i18n>

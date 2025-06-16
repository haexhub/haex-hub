<template>
  <div>
    <HaexPassGroup
      v-model="group"
      mode="create"
      @close="onClose"
      @submit="createAsync"
    />
  </div>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

definePageMeta({
  name: 'passwordGroupCreate',
})

const { currentGroupId } = storeToRefs(usePasswordGroupStore())
const group = ref<SelectHaexPasswordsGroups>({
  name: '',
  description: '',
  id: '',
  color: null,
  icon: null,
  order: null,
  parentId: currentGroupId.value || null,
  createdAt: null,
  updateAt: null,
})

const errors = ref({
  name: [],
  description: [],
})

const onClose = () => {
  useRouter().back()
}

const { addGroupAsync } = usePasswordGroupStore()
const createAsync = async () => {
  try {
    if (errors.value.name.length || errors.value.description.length) return

    const newGroup = await addGroupAsync(group.value)

    console.log('newGroup', newGroup)
    if (!newGroup.id) {
      return
    }

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

<template>
  <UiDialog :title="t('title')">
    <form class="flex flex-col">
      <UiInput v-model="vaultItem.title" />
      <UiInput v-model="vaultItem.username" />
      <UiInput v-model="vaultItem.password" />
      <UiInput v-model="vaultItem.note" />
    </form>
  </UiDialog>
</template>

<script setup lang="ts">
const { t } = useI18n();

const vaultItem = reactive({
  title: '',
  username: '',
  password: '',
  note: '',
});
</script>

<i18n lang="json">
{
  "de": {
    "title": "Eintrag erstellen"
  },
  "en": {
    "title": "Create Entry"
  }
}
</i18n>

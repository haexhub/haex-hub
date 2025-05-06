<template>
  <div>
    <HaexExtensionManifestPermissionsTitle>
      {{ t("database.title") }}
    </HaexExtensionManifestPermissionsTitle>

    <div v-if="database?.read?.length">
      <UiAccordion>
        <template #title>
          <h3>{{ t("permission.read") }}</h3>
        </template>

        <ul class="space-y-0.5">
          <li class="flex items-center justify-between px-4 py-0.5" v-for="read in database?.read">
            <div class="flex items-center gap-2">
              <span>{{ read }}</span>
            </div>
          </li>
        </ul>
      </UiAccordion>
    </div>

    <div v-if="database?.write?.length">
      <UiAccordion>
        <template #title>
          <h3>{{ t("permission.write") }}</h3>
        </template>

        <ul class="space-y-0.5">
          <li
            class="flex items-center justify-between px-4 py-0.5"
            v-for="write in database?.write"
          >
            <div class="flex items-center gap-2">
              <span>{{ write }}</span>
            </div>
          </li>
        </ul>
      </UiAccordion>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ database: IHaexHubExtensionManifest["permissions"]["database"] }>();
const { t } = useI18n();
</script>

<i18n lang="json">
{
  "de": {
    "permission": {
      "read": "Lesen",
      "write": "Schreiben"
    },

    "database": {
      "title": "Datenbank Berechtigungen"
    }
  },
  "en": { "title": "Confirm Permission" }
}
</i18n>

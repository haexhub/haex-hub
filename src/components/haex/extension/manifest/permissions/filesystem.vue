<template>
  <div>
    <HaexExtensionManifestPermissionsTitle>
      {{ t("filesystem.title") }}
    </HaexExtensionManifestPermissionsTitle>

    <div v-if="filesystem?.read?.length">
      <UiAccordion>
        <template #title>
          <h3>{{ t("permission.read") }}</h3>
        </template>
        <ul class="space-y-0.5">
          <li
            class="flex items-center justify-between px-4 py-0.5"
            v-for="read in filesystem?.read"
          >
            <div class="flex items-center gap-2">
              <span>{{ read }}</span>
            </div>
          </li>
        </ul>
      </UiAccordion>
    </div>

    <div v-if="filesystem?.write?.length">
      <UiAccordion>
        <template #title>
          <h3>{{ t("permission.write") }}</h3>
        </template>

        <ul class="space-y-0.5">
          <li
            class="flex items-center justify-between px-4 py-0.5"
            v-for="write in filesystem?.write"
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
defineProps<{ filesystem: IHaexHubExtensionManifest["permissions"]["filesystem"] }>();
const { t } = useI18n();
</script>

<i18n lang="json">
{
  "de": {
    "permission": {
      "read": "Lesen",
      "write": "Schreiben"
    },

    "filesystem": {
      "title": "Dateisystem Berechtigungen"
    }
  },
  "en": { "title": "Confirm Permission" }
}
</i18n>

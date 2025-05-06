<template>
  <div>
    <HaexExtensionManifestPermissionsTitle>
      {{ t("http.title") }}
    </HaexExtensionManifestPermissionsTitle>

    <div v-if="http?.length">
      <UiAccordion>
        <template #title>
          <h3>{{ t("permission.access") }}</h3>
        </template>

        <ul class="space-y-0.5">
          <li class="flex items-center justify-between px-4 py-0.5" v-for="access in http">
            <div class="flex items-center gap-2">
              <span>{{ access }}</span>
            </div>
          </li>
        </ul>
      </UiAccordion>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ http: IHaexHubExtensionManifest["permissions"]["http"] }>();
const { t } = useI18n();
</script>

<i18n lang="json">
{
  "de": {
    "permission": {
      "access": "Zugriff"
    },

    "http": {
      "title": "Internet Berechtigungen"
    }
  },
  "en": { "title": "Confirm Permission" }
}
</i18n>

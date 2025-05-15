export default defineNuxtRouteMiddleware(async (to) => {
  const { openVaults } = storeToRefs(useVaultStore());

  const toVaultId = getSingleRouteParam(to.params.vaultId);

  if (!openVaults.value?.[toVaultId]) {
    return await navigateTo(useLocalePath()({ name: "vaultOpen" }));
  }
});

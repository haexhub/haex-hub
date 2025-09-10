export default defineNuxtRouteMiddleware(async (to) => {
  const { openVaults } = storeToRefs(useVaultStore())

  const toVaultId = getSingleRouteParam(to.params.vaultId)

  console.log('middleware', openVaults.value?.[toVaultId])
  if (!openVaults.value?.[toVaultId]) {
    return await navigateTo(useLocalePath()({ name: 'vaultOpen' }))
  }
})

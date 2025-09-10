<template>
  <UDropdownMenu
    :items="icons"
    class="btn"
    @select="(newIcon) => (iconName = newIcon)"
    :read_only
  >
    <template #activator>
      <Icon :name="iconName ? iconName : defaultIcon || icons.at(0)" />
    </template>

    <template #items="{ items }">
      <div class="grid grid-cols-6 -ml-2">
        <li
          class="dropdown-item"
          v-for="item in items"
          @click="read_only ? '' : (iconName = item)"
        >
          <Icon
            :name="item"
            size="36"
          />
        </li>
      </div>
    </template>
  </UDropdownMenu>
</template>

<script setup lang="ts">
const icons = [
  'mdi:folder-outline',
  'mdi:key-outline',
  'pepicons-pop:smartphone-home-button',
  'mdi:amazon',
  'proicons:bank',
  'mdi:bitcoin',
  'mdi:piggy-bank-outline',
  'mdi:account-outline',
  'proicons:computer',
  'proicons:cloud',
  'proicons:game',
  'proicons:github',
  'proicons:wrench',
  'proicons:vehicle-car',
  'proicons:wi-fi',
  'meteor-icons:microchip',
  'meteor-icons:headphones',
  'meteor-icons:star',
  'fe:mail',
  'fe:rocket',
  'fxemoji:trolleybus',
  'mdi:folder-outline',
  'mdi:key-outline',
  'pepicons-pop:smartphone-home-button',
  'mdi:amazon',
  'proicons:bank',
  'mdi:bitcoin',
  'mdi:piggy-bank-outline',
  'mdi:account-outline',
  'proicons:computer',
  'proicons:cloud',
  'proicons:game',
  'proicons:github',
  'proicons:wrench',
  'proicons:vehicle-car',
  'proicons:wi-fi',
  'meteor-icons:microchip',
  'meteor-icons:headphones',
  'meteor-icons:star',
  'fe:mail',
  'fe:rocket',
  'fxemoji:trolleybus',
]

const iconName = defineModel<string | undefined | null>()

defineProps<{ defaultIcon?: string; read_only?: boolean }>()
</script>

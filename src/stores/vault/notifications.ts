import { eq } from 'drizzle-orm'
import {
  haexNotifications,
  type InsertHaexNotifications,
} from '~~/src-tauri/database/schemas/vault'
import {
  channels,
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification'

export interface IHaexNotification {
  id: string
  title: string | null
  text?: string | null
  icon?: string | null
  image?: string | null
  alt?: string | null
  date: string | null
  type?: 'error' | 'success' | 'warning' | 'info' | null
}

export const useNotificationStore = defineStore('notificationStore', () => {
  const isNotificationAllowed = ref<boolean>(false)

  const requestNotificationPermissionAsync = async () => {
    console.log('requestNotificationPermissionAsync')
    const permission = await requestPermission()
    console.log('got permission', permission)
    isNotificationAllowed.value = permission === 'granted'
    sendNotification({
      title: 'Tauri',
      body: 'Tauri is awesome!',
      icon: 'dialog-information',
    })
    /* const existingChannels = await channels()
    console.log('existingChannels', existingChannels) */
  }
  const test = async () => console.log('test')
  const checkNotificationAsync = async () => {
    isNotificationAllowed.value = await isPermissionGranted()
    return isNotificationAllowed.value
  }

  const notifications = ref<IHaexNotification[]>([])

  const readNotificationsAsync = async (read: boolean = false) => {
    const { currentVault } = storeToRefs(useVaultStore())
    notifications.value = await currentVault.value.drizzle
      .select()
      .from(haexNotifications)
      .where(eq(haexNotifications.read, read))
    console.log('readNotificationsAsync', notifications.value)
  }

  const addNotificationAsync = async (
    notification: Partial<InsertHaexNotifications>,
  ) => {
    const { currentVault } = storeToRefs(useVaultStore())
    try {
      const _notification: InsertHaexNotifications = {
        id: crypto.randomUUID(),
        type: notification.type || 'info',
        alt: notification.alt,
        date: new Date().toUTCString(),
        icon: notification.icon,
        image: notification.image,
        read: notification.read || false,
        text: notification.text ?? '',
        title: notification.title ?? '',
      }

      await currentVault.value.drizzle
        .insert(haexNotifications)
        .values(_notification)

      await readNotificationsAsync()

      if (!isNotificationAllowed.value) {
        const permission = await requestPermission()
        isNotificationAllowed.value = permission === 'granted'
      }

      if (isNotificationAllowed.value) {
        sendNotification({
          title: _notification.title!,
          body: _notification.text!,
        })
      }
    } catch (error) {}
  }

  return {
    notifications,
    isNotificationAllowed,
    checkNotificationAsync,
    addNotificationAsync,
    readNotificationsAsync,
    requestNotificationPermissionAsync,
    test,
  }
})

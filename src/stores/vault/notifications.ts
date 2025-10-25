import { and, eq, or, type SQLWrapper } from 'drizzle-orm'
import {
  haexNotifications,
  type InsertHaexNotifications,
} from '~~/src-tauri/database/schemas/haex'
import {
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
  type?: 'error' | 'success' | 'warning' | 'info' | 'log' | null
}

export const useNotificationStore = defineStore('notificationStore', () => {
  const isNotificationAllowed = ref<boolean>(false)

  const requestNotificationPermissionAsync = async () => {
    console.log('requestNotificationPermissionAsync')
    const permission = await requestPermission()
    console.log('got permission', permission)
    isNotificationAllowed.value = permission === 'granted'
  }

  const checkNotificationAsync = async () => {
    isNotificationAllowed.value = await isPermissionGranted()
    return isNotificationAllowed.value
  }

  const notifications = ref<IHaexNotification[]>([])

  const readNotificationsAsync = async (filter?: SQLWrapper[]) => {
    const { currentVault } = storeToRefs(useVaultStore())

    if (filter) {
      return await currentVault.value?.drizzle
        .select()
        .from(haexNotifications)
        .where(and(...filter))
    } else {
      return await currentVault.value?.drizzle.select().from(haexNotifications)
    }
  }

  const syncNotificationsAsync = async () => {
    notifications.value =
      (await readNotificationsAsync([eq(haexNotifications.read, false)])) ?? []
  }

  const addNotificationAsync = async (
    notification: Partial<InsertHaexNotifications>,
  ) => {
    const { currentVault } = storeToRefs(useVaultStore())
    try {
      const _notification: InsertHaexNotifications = {
        id: crypto.randomUUID(),
        alt: notification.alt,
        date: notification.date || new Date().toUTCString(),
        icon: notification.icon,
        image: notification.image,
        read: notification.read || false,
        source: notification.source,
        text: notification.text,
        title: notification.title,
        type: notification.type || 'info',
      }

      await currentVault.value?.drizzle
        .insert(haexNotifications)
        .values(_notification)

      await syncNotificationsAsync()

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
    } catch (error) {
      console.error(error)
    }
  }

  const deleteNotificationsAsync = async (notificationIds: string[]) => {
    const { currentVault } = storeToRefs(useVaultStore())
    const filter = notificationIds.map((id) => eq(haexNotifications.id, id))

    console.log('deleteNotificationsAsync', notificationIds)
    return currentVault.value?.drizzle
      .delete(haexNotifications)
      .where(or(...filter))
  }

  return {
    addNotificationAsync,
    checkNotificationAsync,
    deleteNotificationsAsync,
    isNotificationAllowed,
    notifications,
    readNotificationsAsync,
    requestNotificationPermissionAsync,
    syncNotificationsAsync,
  }
})

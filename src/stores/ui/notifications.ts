export interface IHaexNotication {
  title: string
  description?: string
  icon?: string
  image?: string
  alt?: string
  date: Date
}

export const useNotificationStore = defineStore('notificationStore', () => {
  const notifications = ref<IHaexNotication[]>([
    {
      title: 'huhu',
      alt: 'test',
      description: 'Ganz was tolles',
      image: 'https://cdn.flyonui.com/fy-assets/avatar/avatar-1.png',
      date: new Date(),
    },
  ])

  return {
    notifications,
  }
})

import { sql } from 'drizzle-orm'
import {
  integer,
  primaryKey,
  sqliteTable,
  text,
  unique,
  type AnySQLiteColumn,
} from 'drizzle-orm/sqlite-core'

export const haexSettings = sqliteTable('haex_settings', {
  id: text().primaryKey(),
  key: text(),
  type: text(),
  value: text(),
})
export type InsertHaexSettings = typeof haexSettings.$inferInsert
export type SelectHaexSettings = typeof haexSettings.$inferSelect

export const haexExtensions = sqliteTable('haex_extensions', {
  id: text().primaryKey(),
  author: text(),
  enabled: integer({ mode: 'boolean' }),
  icon: text(),
  name: text(),
  url: text(),
  version: text(),
})
export type InsertHaexExtensions = typeof haexExtensions.$inferInsert
export type SelectHaexExtensions = typeof haexExtensions.$inferSelect

export const haexExtensionsPermissions = sqliteTable(
  'haex_extensions_permissions',
  {
    id: text().primaryKey(),
    extensionId: text('extension_id').references(
      (): AnySQLiteColumn => haexExtensions.id,
    ),
    resource: text({ enum: ['fs', 'http', 'db', 'shell'] }),
    operation: text({ enum: ['read', 'write', 'create'] }),
    path: text(),
    createdAt: text('created_at').default(sql`(CURRENT_TIMESTAMP)`),
    updateAt: integer('updated_at', { mode: 'timestamp' }).$onUpdate(
      () => new Date(),
    ),
  },
  (table) => [
    unique().on(table.extensionId, table.resource, table.operation, table.path),
  ],
)
export type InsertHaexExtensionsPermissions =
  typeof haexExtensionsPermissions.$inferInsert
export type SelectHaexExtensionsPermissions =
  typeof haexExtensionsPermissions.$inferSelect

export const haexNotifications = sqliteTable('haex_notifications', {
  id: text().primaryKey(),
  alt: text(),
  date: text(),
  icon: text(),
  image: text(),
  read: integer({ mode: 'boolean' }),
  source: text(),
  text: text(),
  title: text(),
  type: text({
    enum: ['error', 'success', 'warning', 'info', 'log'],
  }).notNull(),
})
export type InsertHaexNotifications = typeof haexNotifications.$inferInsert
export type SelectHaexNotifications = typeof haexNotifications.$inferSelect

export const haexPasswordsItemDetails = sqliteTable(
  'haex_passwords_item_details',
  {
    id: text().primaryKey(),
    title: text(),
    username: text(),
    password: text(),
    note: text(),
    icon: text(),
    tags: text(),
    url: text(),
    createdAt: text('created_at').default(sql`(CURRENT_TIMESTAMP)`),
    updateAt: integer('updated_at', { mode: 'timestamp' }).$onUpdate(
      () => new Date(),
    ),
  },
)
export type InsertHaexPasswordsItemDetails =
  typeof haexPasswordsItemDetails.$inferInsert
export type SelectHaexPasswordsItemDetails =
  typeof haexPasswordsItemDetails.$inferSelect

export const haexPasswordsItemKeyValues = sqliteTable(
  'haex_passwords_item_key_values',
  {
    id: text().primaryKey(),
    itemId: text('item_id').references(
      (): AnySQLiteColumn => haexPasswordsItemDetails.id,
    ),
    key: text(),
    value: text(),
    updateAt: integer('updated_at', { mode: 'timestamp' }).$onUpdate(
      () => new Date(),
    ),
  },
)
export type InserthaexPasswordsItemKeyValues =
  typeof haexPasswordsItemKeyValues.$inferInsert
export type SelectHaexPasswordsItemKeyValues =
  typeof haexPasswordsItemKeyValues.$inferSelect

export const haexPasswordsItemHistory = sqliteTable(
  'haex_passwords_item_history',
  {
    id: text().primaryKey(),
    itemId: text('item_id').references(
      (): AnySQLiteColumn => haexPasswordsItemDetails.id,
    ),
    changedProperty:
      text('changed_property').$type<keyof typeof haexPasswordsItemDetails>(),
    oldValue: text('old_value'),
    newValue: text('new_value'),
    createdAt: text('created_at').default(sql`(CURRENT_TIMESTAMP)`),
  },
)
export type InserthaexPasswordsItemHistory =
  typeof haexPasswordsItemHistory.$inferInsert
export type SelectHaexPasswordsItemHistory =
  typeof haexPasswordsItemHistory.$inferSelect

export const haexPasswordsGroups = sqliteTable('haex_passwords_groups', {
  id: text().primaryKey(),
  name: text(),
  description: text(),
  icon: text(),
  order: integer(),
  color: text(),
  parentId: text('parent_id').references(
    (): AnySQLiteColumn => haexPasswordsGroups.id,
  ),
  createdAt: text('created_at').default(sql`(CURRENT_TIMESTAMP)`),
  updateAt: integer('updated_at', { mode: 'timestamp' }).$onUpdate(
    () => new Date(),
  ),
})
export type InsertHaexPasswordsGroups = typeof haexPasswordsGroups.$inferInsert
export type SelectHaexPasswordsGroups = typeof haexPasswordsGroups.$inferSelect

export const haexPasswordsGroupItems = sqliteTable(
  'haex_passwords_group_items',
  {
    groupId: text('group_id').references(
      (): AnySQLiteColumn => haexPasswordsGroups.id,
    ),
    itemId: text('item_id').references(
      (): AnySQLiteColumn => haexPasswordsItemDetails.id,
    ),
  },
  (table) => [primaryKey({ columns: [table.itemId, table.groupId] })],
)
export type InsertHaexPasswordsGroupItems =
  typeof haexPasswordsGroupItems.$inferInsert
export type SelectHaexPasswordsGroupItems =
  typeof haexPasswordsGroupItems.$inferSelect

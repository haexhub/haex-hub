import { sql } from 'drizzle-orm'
import {
  integer,
  primaryKey,
  sqliteTable,
  text,
  type AnySQLiteColumn,
} from 'drizzle-orm/sqlite-core'
import tableNames from '../tableNames.json'

export const haexPasswordsItemDetails = sqliteTable(
  tableNames.haex.passwords.item_details,
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
    haex_tombstone: integer({ mode: 'boolean' }),
  },
)
export type InsertHaexPasswordsItemDetails =
  typeof haexPasswordsItemDetails.$inferInsert
export type SelectHaexPasswordsItemDetails =
  typeof haexPasswordsItemDetails.$inferSelect

export const haexPasswordsItemKeyValues = sqliteTable(
  tableNames.haex.passwords.item_key_values,
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
    haex_tombstone: integer({ mode: 'boolean' }),
  },
)
export type InserthaexPasswordsItemKeyValues =
  typeof haexPasswordsItemKeyValues.$inferInsert
export type SelectHaexPasswordsItemKeyValues =
  typeof haexPasswordsItemKeyValues.$inferSelect

export const haexPasswordsItemHistory = sqliteTable(
  tableNames.haex.passwords.item_histories,
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
    haex_tombstone: integer({ mode: 'boolean' }),
  },
)
export type InserthaexPasswordsItemHistory =
  typeof haexPasswordsItemHistory.$inferInsert
export type SelectHaexPasswordsItemHistory =
  typeof haexPasswordsItemHistory.$inferSelect

export const haexPasswordsGroups = sqliteTable(
  tableNames.haex.passwords.groups,
  {
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
    haex_tombstone: integer({ mode: 'boolean' }),
  },
)
export type InsertHaexPasswordsGroups = typeof haexPasswordsGroups.$inferInsert
export type SelectHaexPasswordsGroups = typeof haexPasswordsGroups.$inferSelect

export const haexPasswordsGroupItems = sqliteTable(
  tableNames.haex.passwords.group_items,
  {
    groupId: text('group_id').references(
      (): AnySQLiteColumn => haexPasswordsGroups.id,
    ),
    itemId: text('item_id').references(
      (): AnySQLiteColumn => haexPasswordsItemDetails.id,
    ),
    haex_tombstone: integer({ mode: 'boolean' }),
  },
  (table) => [primaryKey({ columns: [table.itemId, table.groupId] })],
)
export type InsertHaexPasswordsGroupItems =
  typeof haexPasswordsGroupItems.$inferInsert
export type SelectHaexPasswordsGroupItems =
  typeof haexPasswordsGroupItems.$inferSelect

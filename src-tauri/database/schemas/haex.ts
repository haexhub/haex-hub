import { sql } from 'drizzle-orm'
import {
  integer,
  sqliteTable,
  text,
  unique,
  type AnySQLiteColumn,
} from 'drizzle-orm/sqlite-core'
import tableNames from '../tableNames.json'

export const haexSettings = sqliteTable(tableNames.haex.settings, {
  id: text()
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  key: text(),
  type: text(),
  value: text(),
  haexTombstone: integer('haex_tombstone', { mode: 'boolean' }),
  haexTimestamp: text('haex_timestamp'),
})
export type InsertHaexSettings = typeof haexSettings.$inferInsert
export type SelectHaexSettings = typeof haexSettings.$inferSelect

export const haexExtensions = sqliteTable(tableNames.haex.extensions, {
  id: text()
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  author: text(),
  description: text(),
  entry: text(),
  homepage: text(),
  enabled: integer({ mode: 'boolean' }),
  icon: text(),
  name: text(),
  public_key: text(),
  signature: text(),
  url: text(),
  version: text(),
  haexTombstone: integer('haex_tombstone', { mode: 'boolean' }),
  haexTimestamp: text('haex_timestamp'),
})
export type InsertHaexExtensions = typeof haexExtensions.$inferInsert
export type SelectHaexExtensions = typeof haexExtensions.$inferSelect

export const haexExtensionPermissions = sqliteTable(
  tableNames.haex.extension_permissions,
  {
    id: text()
      .primaryKey()
      .$defaultFn(() => crypto.randomUUID()),
    extensionId: text('extension_id').references(
      (): AnySQLiteColumn => haexExtensions.id,
    ),
    resourceType: text('resource_type', {
      enum: ['fs', 'http', 'db', 'shell'],
    }),
    action: text({ enum: ['read', 'write'] }),
    target: text(),
    constraints: text({ mode: 'json' }),
    status: text({ enum: ['ask', 'granted', 'denied'] })
      .notNull()
      .default('denied'),
    createdAt: text('created_at').default(sql`(CURRENT_TIMESTAMP)`),
    updateAt: integer('updated_at', { mode: 'timestamp' }).$onUpdate(
      () => new Date(),
    ),
    haexTombstone: integer('haex_tombstone', { mode: 'boolean' }),
    haexTimestamp: text('haex_timestamp'),
  },
  (table) => [
    unique().on(
      table.extensionId,
      table.resourceType,
      table.action,
      table.target,
    ),
  ],
)
export type InserthaexExtensionPermissions =
  typeof haexExtensionPermissions.$inferInsert
export type SelecthaexExtensionPermissions =
  typeof haexExtensionPermissions.$inferSelect

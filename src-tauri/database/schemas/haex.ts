import { sql } from 'drizzle-orm'
import {
  integer,
  sqliteTable,
  text,
  unique,
  type AnySQLiteColumn,
} from 'drizzle-orm/sqlite-core'
import tableNames from '../tableNames.json'

export const haexSettings = sqliteTable(tableNames.haex.settings.name, {
  id: text()
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  key: text(),
  type: text(),
  value: text(),
  haexTombstone: integer(tableNames.haex.settings.columns.haexTombstone, {
    mode: 'boolean',
  }),
  haexTimestamp: text(tableNames.haex.settings.columns.haexTimestamp),
})
export type InsertHaexSettings = typeof haexSettings.$inferInsert
export type SelectHaexSettings = typeof haexSettings.$inferSelect

export const haexExtensions = sqliteTable(
  tableNames.haex.extensions.name,
  {
    id: text()
      .primaryKey()
      .$defaultFn(() => crypto.randomUUID()),
    public_key: text().notNull(),
    name: text().notNull(),
    version: text().notNull(),
    author: text(),
    description: text(),
    entry: text().notNull().default('index.html'),
    homepage: text(),
    enabled: integer({ mode: 'boolean' }).default(true),
    icon: text(),
    signature: text().notNull(),
    haexTombstone: integer(tableNames.haex.extensions.columns.haexTombstone, {
      mode: 'boolean',
    }),
    haexTimestamp: text(tableNames.haex.extensions.columns.haexTimestamp),
  },
  (table) => [
    // UNIQUE constraint: Pro Developer (public_key) kann nur eine Extension mit diesem Namen existieren
    unique().on(table.public_key, table.name),
  ],
)
export type InsertHaexExtensions = typeof haexExtensions.$inferInsert
export type SelectHaexExtensions = typeof haexExtensions.$inferSelect

export const haexExtensionPermissions = sqliteTable(
  tableNames.haex.extension_permissions.name,
  {
    id: text()
      .primaryKey()
      .$defaultFn(() => crypto.randomUUID()),
    extensionId: text(
      tableNames.haex.extension_permissions.columns.extensionId,
    ).references((): AnySQLiteColumn => haexExtensions.id),
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
    haexTombstone: integer(
      tableNames.haex.extension_permissions.columns.haexTombstone,
      { mode: 'boolean' },
    ),
    haexTimestamp: text(
      tableNames.haex.extension_permissions.columns.haexTimestamp,
    ),
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

export const haexNotifications = sqliteTable(
  tableNames.haex.notifications.name,
  {
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
    haexTombstone: integer(
      tableNames.haex.notifications.columns.haexTombstone,
      { mode: 'boolean' },
    ),
    haexTimestamp: text(tableNames.haex.notifications.columns.haexTimestamp),
  },
)
export type InsertHaexNotifications = typeof haexNotifications.$inferInsert
export type SelectHaexNotifications = typeof haexNotifications.$inferSelect

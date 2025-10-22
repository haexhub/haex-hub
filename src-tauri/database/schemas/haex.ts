import { sql } from 'drizzle-orm'
import {
  integer,
  sqliteTable,
  text,
  unique,
  type AnySQLiteColumn,
  type SQLiteColumnBuilderBase,
} from 'drizzle-orm/sqlite-core'
import tableNames from '../tableNames.json'

// Helper function to add common CRDT columns (haexTombstone and haexTimestamp)
export const withCrdtColumns = <
  T extends Record<string, SQLiteColumnBuilderBase>,
>(
  columns: T,
  columnNames: { haexTombstone: string; haexTimestamp: string },
) => ({
  ...columns,
  haexTombstone: integer(columnNames.haexTombstone, { mode: 'boolean' }),
  haexTimestamp: text(columnNames.haexTimestamp),
})

export const haexSettings = sqliteTable(
  tableNames.haex.settings.name,
  {
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
  },
  (table) => [unique().on(table.key, table.type, table.value)],
)
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

export const haexWorkspaces = sqliteTable(
  tableNames.haex.workspaces.name,
  withCrdtColumns(
    {
      id: text(tableNames.haex.workspaces.columns.id)
        .primaryKey()
        .$defaultFn(() => crypto.randomUUID()),
      name: text(tableNames.haex.workspaces.columns.name).notNull(),
      position: integer(tableNames.haex.workspaces.columns.position)
        .notNull()
        .default(0),
      createdAt: integer(tableNames.haex.workspaces.columns.createdAt, {
        mode: 'timestamp',
      })
        .notNull()
        .$defaultFn(() => new Date()),
    },
    tableNames.haex.workspaces.columns,
  ),
  (table) => [unique().on(table.position)],
)
export type InsertHaexWorkspaces = typeof haexWorkspaces.$inferInsert
export type SelectHaexWorkspaces = typeof haexWorkspaces.$inferSelect

export const haexDesktopItems = sqliteTable(
  tableNames.haex.desktop_items.name,
  withCrdtColumns(
    {
      id: text(tableNames.haex.desktop_items.columns.id)
        .primaryKey()
        .$defaultFn(() => crypto.randomUUID()),
      workspaceId: text(tableNames.haex.desktop_items.columns.workspaceId)
        .notNull()
        .references(() => haexWorkspaces.id),
      itemType: text(tableNames.haex.desktop_items.columns.itemType, {
        enum: ['extension', 'file', 'folder'],
      }).notNull(),
      referenceId: text(
        tableNames.haex.desktop_items.columns.referenceId,
      ).notNull(), // extensionId für extensions, filePath für files/folders
      positionX: integer(tableNames.haex.desktop_items.columns.positionX)
        .notNull()
        .default(0),
      positionY: integer(tableNames.haex.desktop_items.columns.positionY)
        .notNull()
        .default(0),
    },
    tableNames.haex.desktop_items.columns,
  ),
)
export type InsertHaexDesktopItems = typeof haexDesktopItems.$inferInsert
export type SelectHaexDesktopItems = typeof haexDesktopItems.$inferSelect

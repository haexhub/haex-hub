import {
  integer,
  sqliteTable,
  text,
  unique,
  type AnySQLiteColumn,
} from 'drizzle-orm/sqlite-core'

export const haexSettings = sqliteTable('haex_settings', {
  id: text().primaryKey(),
  key: text(),
  value: text(),
})

export const haexExtensions = sqliteTable('haex_extensions', {
  id: text().primaryKey(),
  name: text(),
  author: text(),
  enabled: integer({ mode: 'boolean' }),
  icon: text(),
  url: text(),
  version: text(),
})

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
  },
  (table) => [
    unique().on(table.extensionId, table.resource, table.operation, table.path),
  ],
)

export type InsertHaexSettings = typeof haexSettings.$inferInsert
export type SelectHaexSettings = typeof haexSettings.$inferSelect

export type InsertHaexExtensions = typeof haexExtensions.$inferInsert
export type SelectHaexExtensions = typeof haexExtensions.$inferSelect

export type InsertHaexExtensionsPermissions =
  typeof haexExtensionsPermissions.$inferInsert
export type SelectHaexExtensionsPermissions =
  typeof haexExtensionsPermissions.$inferSelect

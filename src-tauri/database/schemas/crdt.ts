import { blob, integer, sqliteTable, text } from 'drizzle-orm/sqlite-core'

export const haexCrdtMessages = sqliteTable('haex_crdt_messages', {
  hlc_timestamp: text().primaryKey(),
  table_name: text(),
  row_pks: text({ mode: 'json' }),
  op_type: text({ enum: ['INSERT', 'UPDATE', 'DELETE'] }),
  column_name: text(),
  new_value: blob(),
  old_value: blob(),
})
export type InsertHaexCrdtMessages = typeof haexCrdtMessages.$inferInsert
export type SelectHaexCrdtMessages = typeof haexCrdtMessages.$inferSelect

export const haexCrdtSnapshots = sqliteTable('haex_crdt_snapshots', {
  snapshot_id: text().primaryKey(),
  created: text(),
  epoch_hlc: text(),
  location_url: text(),
  file_size_bytes: integer(),
})

import { integer, sqliteTable, text } from 'drizzle-orm/sqlite-core'
import tableNames from '../tableNames.json'

export const haexCrdtLogs = sqliteTable(tableNames.haex.crdt.logs, {
  hlc_timestamp: text().primaryKey(),
  table_name: text(),
  row_pks: text({ mode: 'json' }),
  op_type: text({ enum: ['INSERT', 'UPDATE', 'DELETE'] }),
  column_name: text(),
  new_value: text({ mode: 'json' }),
  old_value: text({ mode: 'json' }),
})
export type InsertHaexCrdtLogs = typeof haexCrdtLogs.$inferInsert
export type SelectHaexCrdtLogs = typeof haexCrdtLogs.$inferSelect

export const haexCrdtSnapshots = sqliteTable(tableNames.haex.crdt.snapshots, {
  snapshot_id: text().primaryKey(),
  created: text(),
  epoch_hlc: text(),
  location_url: text(),
  file_size_bytes: integer(),
})

export const haexCrdtConfigs = sqliteTable(tableNames.haex.crdt.configs, {
  key: text().primaryKey(),
  value: text(),
})

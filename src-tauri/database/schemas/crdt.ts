import { integer, sqliteTable, text, index } from 'drizzle-orm/sqlite-core'
import tableNames from '../tableNames.json'

export const haexCrdtLogs = sqliteTable(
  tableNames.haex.crdt.logs,
  {
    id: text()
      .primaryKey()
      .$defaultFn(() => crypto.randomUUID()),
    haexTimestamp: text('haex_timestamp'),
    tableName: text('table_name'),
    rowPks: text('row_pks', { mode: 'json' }),
    opType: text('op_type', { enum: ['INSERT', 'UPDATE', 'DELETE'] }),
    columnName: text('column_name'),
    newValue: text('new_value', { mode: 'json' }),
    oldValue: text('old_value', { mode: 'json' }),
  },
  (table) => [
    index('idx_haex_timestamp').on(table.haexTimestamp),
    index('idx_table_row').on(table.tableName, table.rowPks),
  ],
)
export type InsertHaexCrdtLogs = typeof haexCrdtLogs.$inferInsert
export type SelectHaexCrdtLogs = typeof haexCrdtLogs.$inferSelect

export const haexCrdtSnapshots = sqliteTable(tableNames.haex.crdt.snapshots, {
  snapshot_id: text()
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  created: text(),
  epoch_hlc: text(),
  location_url: text(),
  file_size_bytes: integer(),
})

export const haexCrdtConfigs = sqliteTable(tableNames.haex.crdt.configs, {
  key: text()
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  value: text(),
})

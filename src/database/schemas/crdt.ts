import { integer, sqliteTable, text, index } from 'drizzle-orm/sqlite-core'
import tableNames from '~/database/tableNames.json'

export const haexCrdtLogs = sqliteTable(
  tableNames.haex.crdt.logs.name,
  {
    id: text()
      .$defaultFn(() => crypto.randomUUID())
      .primaryKey(),
    haexTimestamp: text(tableNames.haex.crdt.logs.columns.haexTimestamp),
    tableName: text(tableNames.haex.crdt.logs.columns.tableName),
    rowPks: text(tableNames.haex.crdt.logs.columns.rowPks, { mode: 'json' }),
    opType: text(tableNames.haex.crdt.logs.columns.opType, {
      enum: ['INSERT', 'UPDATE', 'DELETE'],
    }),
    columnName: text(tableNames.haex.crdt.logs.columns.columnName),
    newValue: text(tableNames.haex.crdt.logs.columns.newValue, {
      mode: 'json',
    }),
    oldValue: text(tableNames.haex.crdt.logs.columns.oldValue, {
      mode: 'json',
    }),
  },
  (table) => [
    index('idx_haex_timestamp').on(table.haexTimestamp),
    index('idx_table_row').on(table.tableName, table.rowPks),
  ],
)
export type InsertHaexCrdtLogs = typeof haexCrdtLogs.$inferInsert
export type SelectHaexCrdtLogs = typeof haexCrdtLogs.$inferSelect

export const haexCrdtSnapshots = sqliteTable(
  tableNames.haex.crdt.snapshots.name,
  {
    snapshotId: text(tableNames.haex.crdt.snapshots.columns.snapshotId)
      .$defaultFn(() => crypto.randomUUID())
      .primaryKey(),
    created: text(),
    epochHlc: text(tableNames.haex.crdt.snapshots.columns.epochHlc),
    locationUrl: text(tableNames.haex.crdt.snapshots.columns.locationUrl),
    fileSizeBytes: integer(
      tableNames.haex.crdt.snapshots.columns.fileSizeBytes,
    ),
  },
)

export const haexCrdtConfigs = sqliteTable(tableNames.haex.crdt.configs.name, {
  key: text().primaryKey(),
  value: text(),
})

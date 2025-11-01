import { writeFileSync, mkdirSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import tablesNames from '../../src/database/tableNames.json'
import { schema } from '../../src/database/index'
import { getTableColumns } from 'drizzle-orm'
import type { AnySQLiteColumn, SQLiteTable } from 'drizzle-orm/sqlite-core'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

interface Column {
  name: string
  rustType: string
  isOptional: boolean
}

function drizzleToRustType(colDef: AnySQLiteColumn): {
  rustType: string
  isOptional: boolean
} {
  let baseType = 'String'
  let isOptional = !colDef.notNull

  if (colDef.columnType === 'SQLiteText') {
    if ('mode' in colDef && colDef.mode === 'json') {
      baseType = 'serde_json::Value'
    } else {
      baseType = 'String'
    }
  } else if (colDef.columnType === 'SQLiteInteger') {
    baseType = 'i64'
  } else if (colDef.columnType === 'SQLiteBoolean') {
    baseType = 'bool'
  } else if (colDef.columnType === 'SQLiteReal') {
    baseType = 'f64'
  } else if (colDef.columnType === 'SQLiteBlob') {
    baseType = 'Vec<u8>'
  }

  // Drizzle verwendet 'primary' für den Primärschlüssel-Status
  if (colDef.primary) {
    isOptional = false
  }

  return { rustType: baseType, isOptional }
}

function extractColumns(table: SQLiteTable): Column[] {
  const columns: Column[] = []

  // getTableColumns gibt ein Record<string, AnySQLiteColumn> zurück
  const tableColumns = getTableColumns(table)

  // Object.values gibt uns ein Array vom Typ AnySQLiteColumn[]
  for (const colDef of Object.values(tableColumns)) {
    // Die relevanten Infos stehen im 'config' Property der Spalte.
    // TypeScript kennt den Typ von 'config' bereits!
    const { rustType, isOptional } = drizzleToRustType(colDef)

    columns.push({
      name: colDef.name,
      rustType: isOptional ? `Option<${rustType}>` : rustType,
      isOptional,
    })
  }
  return columns
}

function toSnakeCase(str: string): string {
  return str.replace(/[A-Z]/g, (letter, index) =>
    index === 0 ? letter.toLowerCase() : `_${letter.toLowerCase()}`,
  )
}

function toPascalCase(str: string): string {
  console.log('toPascalCase:', str)
  return str
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join('')
}

const RUST_KEYWORDS = new Set([
  'type',
  'struct',
  'enum',
  'pub',
  'use',
  'as',
  'crate',
  'super',
  'self',
  'let',
  'mut',
])

function generateStruct(name: string, columns: Column[]): string {
  let structName = toPascalCase(name)

  if (RUST_KEYWORDS.has(structName.toLowerCase())) {
    structName = `r#${structName}`
  }

  // --- Teil 1: Struct-Definition ---
  let code = `#[derive(Debug, Clone, Serialize, Deserialize)]\n`
  code += `#[serde(rename_all = "camelCase")]\n`
  code += `pub struct ${structName} {\n`

  for (const col of columns) {
    let fieldName = toSnakeCase(col.name)

    // Prüfen, ob der Name ein Keyword ist
    if (RUST_KEYWORDS.has(fieldName)) {
      fieldName = `r#${fieldName}`
    }

    if (col.isOptional) {
      code += `    #[serde(skip_serializing_if = "Option::is_none")]\n`
    }
    // Wichtig: #[serde(rename = "...")] hinzufügen, falls der Feldname geändert wurde!
    if (fieldName.startsWith('r#')) {
      const originalName = fieldName.substring(2)
      code += `    #[serde(rename = "${originalName}")]\n`
    }
    code += `    pub ${fieldName}: ${col.rustType},\n`
  }

  code += `}\n\n`

  // --- Teil 2: Impl-Block ---
  code += `impl ${structName} {\n`
  code += `    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {\n`
  code += `        Ok(Self {\n`

  columns.forEach((col, idx) => {
    let fieldName = toSnakeCase(col.name)
    if (RUST_KEYWORDS.has(fieldName)) {
      fieldName = `r#${fieldName}`
    }
    code += `            ${fieldName}: row.get(${idx})?,\n`
  })

  code += `        })\n`
  code += `    }\n`
  code += `}\n\n`

  return code
}

function main() {
  let output = `// Auto-generated from Drizzle schema
// DO NOT EDIT MANUALLY
// Run 'pnpm generate:rust-types' to regenerate

use serde::{Deserialize, Serialize};

`

  const schemas = [
    { name: tablesNames.haex.settings.name, table: schema.haexSettings },
    { name: tablesNames.haex.extensions.name, table: schema.haexExtensions },
    {
      name: tablesNames.haex.extension_permissions.name,
      table: schema.haexExtensionPermissions,
    },
    { name: tablesNames.haex.crdt.logs.name, table: schema.haexCrdtLogs },
    {
      name: tablesNames.haex.crdt.snapshots.name,
      table: schema.haexCrdtSnapshots,
    },
    { name: tablesNames.haex.crdt.configs.name, table: schema.haexCrdtConfigs },
    {
      name: tablesNames.haex.desktop_items.name,
      table: schema.haexDesktopItems,
    },
    {
      name: tablesNames.haex.workspaces.name,
      table: schema.haexWorkspaces,
    },
  ]

  for (const { name, table } of schemas) {
    console.log(`\n=== Processing table: ${name} ===`)
    const columns = extractColumns(table)
    console.log(`Found ${columns.length} columns`)

    if (columns.length > 0) {
      output += generateStruct(name, columns)
    }
  }

  const outputPath = join(__dirname, '../src/database/generated.rs')
  mkdirSync(dirname(outputPath), { recursive: true })
  writeFileSync(outputPath, output, 'utf-8')

  console.log('\n✅ Rust types generated:', outputPath)
}

main()

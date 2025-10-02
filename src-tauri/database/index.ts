import { drizzle } from 'drizzle-orm/sqlite-proxy' // Adapter für Query Building ohne direkte Verbindung
import * as schema from './schemas' // Importiere alles aus deiner Schema-Datei
export * as schema from './schemas'
// sqlite-proxy benötigt eine (dummy) Ausführungsfunktion als Argument.
// Diese wird in unserem Tauri-Workflow nie aufgerufen, da wir nur .toSQL() verwenden.
// Sie muss aber vorhanden sein, um drizzle() aufrufen zu können.
const dummyExecutor = async (
  sql: string,
  params: unknown[],
  method: 'all' | 'run' | 'get' | 'values',
) => {
  console.warn(
    `Frontend Drizzle Executor wurde aufgerufen (Methode: ${method}). Das sollte im Tauri-Invoke-Workflow nicht passieren!`,
  )
  // Wir geben leere Ergebnisse zurück, um die Typen zufriedenzustellen, falls es doch aufgerufen wird.
  return { rows: [] } // Für 'run' (z.B. bei INSERT/UPDATE)
}

// Erstelle die Drizzle-Instanz für den SQLite-Dialekt
// Übergib den dummyExecutor und das importierte Schema
export const db = drizzle(dummyExecutor, { schema })

// Exportiere auch alle Schema-Definitionen weiter, damit man alles aus einer Datei importieren kann

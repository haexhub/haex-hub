import { defineConfig } from 'drizzle-kit'

export default defineConfig({
  schema: './src/database/schemas/**.ts',
  out: './src-tauri/database/migrations',
  dialect: 'sqlite',
  dbCredentials: {
    url: './src-tauri/database/vault.db',
  },
})

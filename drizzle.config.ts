import { defineConfig } from 'drizzle-kit';

export default defineConfig({
  schema: './src/database/schemas/**.ts',
  out: './src/database/migrations',
  dialect: 'sqlite',
  dbCredentials: {
    url: './src/database/default.db',
  },
});

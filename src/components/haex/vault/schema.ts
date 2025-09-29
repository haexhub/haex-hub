import { z } from 'zod'

export const vaultSchema = {
  password: z.string().min(6).max(255),
  name: z.string().min(1).max(255),
  path: z.string().min(4).endsWith('.db'),
}

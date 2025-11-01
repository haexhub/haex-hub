#!/usr/bin/env node

/**
 * Post-build script to fix the Vite 7.x TDZ error in __vite__mapDeps
 * This script patches the generated JavaScript files after the build
 */

import { readdir, readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const NUXT_DIR = join(process.cwd(), '.output/public/_nuxt')

async function fixFile(filePath) {
  const content = await readFile(filePath, 'utf-8')
  const fixedContent = content.replace(
    /const __vite__mapDeps=\(i,m=__vite__mapDeps,/g,
    'let __vite__mapDeps;__vite__mapDeps=(i,m=__vite__mapDeps,'
  )

  if (fixedContent !== content) {
    await writeFile(filePath, fixedContent, 'utf-8')
    console.log(`✓ Fixed TDZ error in ${filePath.split('/').pop()}`)
    return true
  }

  return false
}

async function main() {
  try {
    const files = await readdir(NUXT_DIR)
    const jsFiles = files.filter((f) => f.endsWith('.js'))

    let fixedCount = 0
    for (const file of jsFiles) {
      const filePath = join(NUXT_DIR, file)
      const fixed = await fixFile(filePath)
      if (fixed) fixedCount++
    }

    if (fixedCount > 0) {
      console.log(`\n✓ Fixed __vite__mapDeps TDZ error in ${fixedCount} file(s)`)
    } else {
      console.log('\n✓ No __vite__mapDeps TDZ errors found')
    }
  } catch (error) {
    console.error('Error fixing __vite__mapDeps:', error)
    process.exit(1)
  }
}

main()

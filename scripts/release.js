#!/usr/bin/env node

import { readFileSync, writeFileSync } from 'fs';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');

const versionType = process.argv[2];

if (!['patch', 'minor', 'major'].includes(versionType)) {
  console.error('Usage: pnpm release <patch|minor|major>');
  process.exit(1);
}

// Read current package.json
const packageJsonPath = join(rootDir, 'package.json');
const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf8'));
const currentVersion = packageJson.version;

if (!currentVersion) {
  console.error('No version found in package.json');
  process.exit(1);
}

// Parse version
const [major, minor, patch] = currentVersion.split('.').map(Number);

// Calculate new version
let newVersion;
switch (versionType) {
  case 'major':
    newVersion = `${major + 1}.0.0`;
    break;
  case 'minor':
    newVersion = `${major}.${minor + 1}.0`;
    break;
  case 'patch':
    newVersion = `${major}.${minor}.${patch + 1}`;
    break;
}

console.log(`📦 Bumping version from ${currentVersion} to ${newVersion}`);

// Update package.json
packageJson.version = newVersion;
writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
console.log('✅ Updated package.json');

// Git operations
try {
  // Check if there are uncommitted changes
  const status = execSync('git status --porcelain', { encoding: 'utf8' });
  const hasOtherChanges = status
    .split('\n')
    .filter(line => line && !line.includes('package.json'))
    .length > 0;

  if (hasOtherChanges) {
    console.error('❌ There are uncommitted changes besides package.json. Please commit or stash them first.');
    process.exit(1);
  }

  // Add and commit package.json
  execSync('git add package.json', { stdio: 'inherit' });
  execSync(`git commit -m "Bump version to ${newVersion}"`, { stdio: 'inherit' });
  console.log('✅ Committed version bump');

  // Create tag
  execSync(`git tag v${newVersion}`, { stdio: 'inherit' });
  console.log(`✅ Created tag v${newVersion}`);

  // Push changes and tag
  console.log('📤 Pushing to remote...');
  execSync('git push', { stdio: 'inherit' });
  execSync(`git push origin v${newVersion}`, { stdio: 'inherit' });
  console.log('✅ Pushed changes and tag');

  console.log('\n🎉 Release v' + newVersion + ' created successfully!');
  console.log('📋 GitHub Actions will now build and publish the release.');
} catch (error) {
  console.error('❌ Git operation failed:', error.message);
  // Rollback package.json changes
  packageJson.version = currentVersion;
  writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
  console.log('↩️  Rolled back package.json changes');
  process.exit(1);
}

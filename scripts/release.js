#!/usr/bin/env node

import { readFileSync, writeFileSync } from 'fs';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, '..');

// Read current versions
const packageJsonPath = join(rootDir, 'package.json');
const tauriConfPath = join(rootDir, 'src-tauri', 'tauri.conf.json');

const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'));
const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf-8'));

const currentVersion = packageJson.version;
const versionParts = currentVersion.split('.').map(Number);

// Increment patch version
versionParts[2]++;
const newVersion = versionParts.join('.');

console.log(`\n🍑 PeachLeaf Release Script`);
console.log(`━━━━━━━━━━━━━━━━━━━━━━━━━━━`);
console.log(`Current version: ${currentVersion}`);
console.log(`New version:     ${newVersion}\n`);

// Update package.json
packageJson.version = newVersion;
writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
console.log(`✓ Updated package.json`);

// Update tauri.conf.json
tauriConf.version = newVersion;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');
console.log(`✓ Updated tauri.conf.json`);

// Git commands
try {
  execSync('git add package.json src-tauri/tauri.conf.json', { cwd: rootDir, stdio: 'inherit' });
  execSync(`git commit -m "chore: bump version to ${newVersion}"`, { cwd: rootDir, stdio: 'inherit' });
  console.log(`✓ Committed version bump`);

  execSync(`git tag v${newVersion}`, { cwd: rootDir, stdio: 'inherit' });
  console.log(`✓ Created tag v${newVersion}`);

  execSync('git push', { cwd: rootDir, stdio: 'inherit' });
  execSync('git push --tags', { cwd: rootDir, stdio: 'inherit' });
  console.log(`✓ Pushed to remote`);

  console.log(`\n🚀 Release v${newVersion} triggered!`);
  console.log(`   Check: https://github.com/hada0127/peach-leaf/actions\n`);
} catch (error) {
  console.error('\n❌ Release failed:', error.message);
  process.exit(1);
}

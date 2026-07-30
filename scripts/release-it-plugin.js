#!/usr/bin/env node
/**
 * Sync version across all files for release-it
 * Usage: node scripts/release-it-plugin.js <version>
 */
import { readFileSync, writeFileSync } from 'fs';

const version = process.argv[2];
if (!version) {
  console.error('Usage: node scripts/release-it-plugin.js <version>');
  process.exit(1);
}

console.log(`Syncing version ${version} across all files...`);

// Update package.json
const pkgPath = 'package.json';
try {
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
  pkg.version = version;
  writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`✓ Updated ${pkgPath}`);
} catch (err) {
  throw new Error(`Failed to update ${pkgPath}: ${err}`);
}

// Update Cargo.toml - match only version = "X.Y.Z" at start of line
const cargoPath = 'src-tauri/Cargo.toml';
try {
  let cargo = readFileSync(cargoPath, 'utf8');
  // Match: version = "X.Y.Z" or version = "X.Y.Z" # comment
  cargo = cargo.replace(/^version\s*=\s*"[^"]+"(\s*#.*)?$/m, `version = "${version}"`);
  writeFileSync(cargoPath, cargo);
  console.log(`✓ Updated ${cargoPath}`);
} catch (err) {
  throw new Error(`Failed to update ${cargoPath}: ${err}`);
}

// Update tauri.conf.json
const tauriPath = 'src-tauri/tauri.conf.json';
try {
  const tauri = JSON.parse(readFileSync(tauriPath, 'utf8'));
  tauri.version = version;
  writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + '\n');
  console.log(`✓ Updated ${tauriPath}`);
} catch (err) {
  throw new Error(`Failed to update ${tauriPath}: ${err}`);
}

console.log(`✅ All files synced to ${version}`);

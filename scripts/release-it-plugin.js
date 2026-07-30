/**
 * release-it plugin to sync version across all files:
 * - package.json
 * - src-tauri/Cargo.toml
 * - src-tauri/tauri.conf.json
 */
import { readFileSync, writeFileSync } from 'fs';

export default async function (_pluginConfig, { version }) {
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

  // Update Cargo.toml
  const cargoPath = 'src-tauri/Cargo.toml';
  try {
    let cargo = readFileSync(cargoPath, 'utf8');
    cargo = cargo.replace(
      /^version = "[^"]+"/m,
      `version = "${version}"                                                              # Synced from package.json`
    );
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
}

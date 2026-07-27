// Update version in config files based on version.txt
import { readFileSync, writeFileSync } from "fs";
import { resolve } from "path";

const scriptDir = new URL(".", import.meta.url).pathname;
const rootDir = resolve(scriptDir, "..");
const versionFile = resolve(rootDir, "version.txt");
const packageJsonPath = resolve(rootDir, "package.json");
const cargoTomlPath = resolve(rootDir, "src-tauri", "Cargo.toml");
const tauriConfigPath = resolve(rootDir, "src-tauri", "tauri.conf.json");

// Read version from version.txt
const version = readFileSync(versionFile, "utf8").trim();

if (!version) {
  throw new Error("version.txt is empty");
}

console.log(`Updating version to: ${version}`);

// Update package.json
let packageJson;
try {
  packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
} catch {
  throw new Error(`Failed to parse ${packageJsonPath}`);
}
packageJson.version = version;
writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + "\n");
console.log(`Updated package.json to ${version}`);

// Update Cargo.toml
const cargoToml = readFileSync(cargoTomlPath, "utf8");
const updatedCargoToml = cargoToml.replace(
  /^version = ".*?"(?:\s+#.*)?$/m,
  `version = "${version}"  # Synced from package.json`
);
writeFileSync(cargoTomlPath, updatedCargoToml);
console.log(`Updated Cargo.toml to ${version}`);

// Update tauri.conf.json
let tauriConfig;
try {
  tauriConfig = JSON.parse(readFileSync(tauriConfigPath, "utf8"));
} catch {
  throw new Error(`Failed to parse ${tauriConfigPath}`);
}
tauriConfig.version = version;
writeFileSync(tauriConfigPath, `${JSON.stringify(tauriConfig, null, 2)}\n`);
console.log(`Updated tauri.conf.json to ${version}`);

console.log("Version update complete!");

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(scriptDir, "..");
const packageJsonPath = resolve(rootDir, "package.json");
const cargoTomlPath = resolve(rootDir, "src-tauri", "Cargo.toml");
const tauriConfigPath = resolve(rootDir, "src-tauri", "tauri.conf.json");

const { version } = JSON.parse(readFileSync(packageJsonPath, "utf8"));

if (!version) {
	throw new Error("package.json version is missing");
}

const cargoToml = readFileSync(cargoTomlPath, "utf8");
const nextCargoToml = cargoToml.replace(
	/^version = ".*?"(?:\s+#.*)?$/m,
	`version = "${version}"  # Synced from package.json`,
);

if (cargoToml === nextCargoToml) {
	console.log(`Cargo.toml already synced to ${version}`);
} else {
	writeFileSync(cargoTomlPath, nextCargoToml);
	console.log(`Updated Cargo.toml to ${version}`);
}

const tauriConfig = JSON.parse(readFileSync(tauriConfigPath, "utf8"));
if (tauriConfig.version !== version) {
	tauriConfig.version = version;
	writeFileSync(tauriConfigPath, `${JSON.stringify(tauriConfig, null, 2)}\n`);
	console.log(`Updated tauri.conf.json to ${version}`);
} else {
	console.log(`tauri.conf.json already synced to ${version}`);
}

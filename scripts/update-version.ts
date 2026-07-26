#!/usr/bin/env bun
// Updates version in all config files from version.txt

import { readFileSync, writeFileSync } from "fs";

const VERSION_FILE = "./version.txt";
const VERSION = readFileSync(VERSION_FILE, "utf-8").trim();
const MAJOR_MINOR = VERSION.split(".").slice(0, 2).join(".");

// Update Cargo.toml
let cargoToml = readFileSync("./src-tauri/Cargo.toml", "utf-8");
cargoToml = cargoToml.replace(/^version = "[\d.]+"$/m, `version = "${VERSION}"`);
writeFileSync("./src-tauri/Cargo.toml", cargoToml);
console.log(`Updated src-tauri/Cargo.toml to ${VERSION}`);

// Update package.json
const packageJson = readFileSync("./package.json", "utf-8");
const pkg = JSON.parse(packageJson);
pkg.version = VERSION;
writeFileSync("./package.json", JSON.stringify(pkg, null, 2) + "\n");
console.log(`Updated package.json to ${VERSION}`);

// Also update src-tauri/tauri.conf.json if it has version
try {
  const tauriConf = readFileSync("./src-tauri/tauri.conf.json", "utf-8");
  const conf = JSON.parse(tauriConf);
  if (conf.package?.version) {
    conf.package.version = VERSION;
    writeFileSync("./src-tauri/tauri.conf.json", JSON.stringify(conf, null, 2) + "\n");
    console.log(`Updated src-tauri/tauri.conf.json to ${VERSION}`);
  }
} catch (e) {
  // tauri.conf.json might not exist or not have version
}

console.log(`\nAll version files updated to ${VERSION}`);

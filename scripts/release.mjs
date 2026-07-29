#!/usr/bin/env bun
/**
 * Companion Release Script
 * Usage: bun scripts/release.sh [patch|minor|major]
 */

import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const scriptDir = import.meta.dirname;
const rootDir = resolve(scriptDir, "..");

const bumpType = process.argv[2] || "patch";

console.log(`🚀 Releasing AMOS Companion (${bumpType})...`);

// Step 1: Sync version first
console.log("🔄 Syncing version to Cargo.toml and tauri.conf.json...");
execSync("bun run sync:version", { cwd: rootDir, stdio: "inherit" });

// Step 2: Run standard-version to bump version and create tag
console.log("📦 Running standard-version...");
const pkg = JSON.parse(readFileSync(resolve(rootDir, "package.json"), "utf8"));
const oldVersion = pkg.version;

// standard-version --release-as bumps version and creates conventional commit
execSync(`bun exec standard-version --release-as ${bumpType} --skip.changelog`, { 
  cwd: rootDir, 
  stdio: "inherit" 
});

// Get new version
const newPkg = JSON.parse(readFileSync(resolve(rootDir, "package.json"), "utf8"));
const newVersion = newPkg.version;

// Step 3: Rename tag to companion/v*
const tag = `companion/v${newVersion}`;
console.log(`🏷️  Renaming tag to: ${tag}`);

// Get current commit hash
const commitHash = execSync("git rev-parse HEAD", { cwd: rootDir, encoding: "utf8" }).trim();

// Delete old tag if exists
try {
  execSync(`git tag -d ${tag}`, { cwd: rootDir });
} catch {}

// Create new annotated tag
execSync(`git tag -a ${tag} -m "Release ${tag}" ${commitHash}`, { cwd: rootDir });

// Step 4: Push
console.log("📤 Pushing to origin...");
execSync("git push origin main --follow-tags", { cwd: rootDir, stdio: "inherit" });

console.log(`
✅ Release ${tag} triggered!
   Old version: ${oldVersion}
   New version: ${newVersion}
   Check build: https://github.com/ManotLuijiu/amos-companion/actions
`);

#!/usr/bin/env bun
/**
 * Bump version script - compatible with bun
 * Usage: bun scripts/bump-version.mjs [patch|minor|major]
 */

import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const scriptDir = import.meta.dirname;
const rootDir = resolve(scriptDir, "..");

function bumpVersion(current, type) {
  const [major, minor, patch] = current.split(".").map(Number);
  
  switch (type) {
    case "major":
      return `${major + 1}.0.0`;
    case "minor":
      return `${major}.${minor + 1}.0`;
    case "patch":
    default:
      return `${major}.${minor}.${patch + 1}`;
  }
}

const args = process.argv.slice(2);
const bumpType = args[0] || "patch";

const packageJsonPath = resolve(rootDir, "package.json");
let pkg;
try {
  pkg = JSON.parse(readFileSync(packageJsonPath, "utf8"));
} catch (e) {
  console.error(`Failed to read package.json: ${e.message}`);
  process.exit(1);
}
const newVersion = bumpVersion(pkg.version, bumpType);

pkg.version = newVersion;
writeFileSync(packageJsonPath, JSON.stringify(pkg, null, 2) + "\n");

console.log(`Version bumped: ${pkg.version} -> ${newVersion}`);

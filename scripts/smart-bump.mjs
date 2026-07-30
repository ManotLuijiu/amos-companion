#!/usr/bin/env bun
/**
 * Smart version bump - analyzes commits to determine major/minor/patch
 * 
 * Usage: bun run smart-bump
 */

import { execSync } from "child_process";

let LAST_TAG;
try {
  LAST_TAG = execSync("git describe --tags --abbrev=0", { encoding: "utf8" }).trim();
} catch {
  console.log("No tags found. Using 0.0.0 as base.");
  LAST_TAG = "v0.0.0";
}

const CURRENT_VERSION = LAST_TAG.replace(/^[a-zA-Z/-]+/, "").replace(/^v/, "");
const [major, minor, patch] = CURRENT_VERSION.split(".").map(Number);

console.log(`Current version: ${CURRENT_VERSION}`);
console.log(`Last tag: ${LAST_TAG}\n`);

// Get commits since last tag
let commits = [];
try {
  const commitOutput = execSync(`git log ${LAST_TAG}..HEAD --oneline --format="%s"`, { encoding: "utf8" });
  commits = commitOutput.trim().split("\n").filter(Boolean);
} catch {
  commits = [];
}

console.log(`Commits since ${LAST_TAG}:`);
if (commits.length === 0) {
  console.log("  (none)");
} else {
  commits.forEach((c, i) => console.log(`  ${i + 1}. ${c}`));
}
console.log("");

// Analyze commits for version bump
let hasBreaking = false;
let hasNewFeature = false;
let hasBugFix = false;

for (const commit of commits) {
  const msg = commit.toLowerCase();
  
  // Breaking changes
  if (msg.includes("breaking") || msg.includes("!:") || msg.includes("breaking change")) {
    hasBreaking = true;
  }
  
  // New features (feat, feature, add, new)
  if (msg.startsWith("feat") || msg.includes(" add ") || msg.includes("new ")) {
    hasNewFeature = true;
  }
  
  // Bug fixes
  if (msg.startsWith("fix")) {
    hasBugFix = true;
  }
}

// Determine bump type
let bumpType = "patch";
let reason = "";

if (hasBreaking) {
  bumpType = "major";
  reason = "Breaking changes detected";
} else if (hasNewFeature) {
  bumpType = "minor";
  reason = "New features added";
} else if (hasBugFix) {
  bumpType = "patch";
  reason = "Bug fixes only";
} else {
  reason = "No significant changes";
}

console.log("=== Version Bump Analysis ===");
console.log(`Breaking changes: ${hasBreaking ? "❌ YES" : "✅ No"}`);
console.log(`New features:    ${hasNewFeature ? "✅ YES" : "❌ No"}`);
console.log(`Bug fixes:       ${hasBugFix ? "✅ YES" : "❌ No"}`);
console.log("");
console.log(`Recommendation: ${bumpType.toUpperCase()}`);
console.log(`Reason: ${reason}`);
console.log("");

// Calculate new version
let newVersion = CURRENT_VERSION;
if (bumpType === "major") {
  newVersion = `${major + 1}.0.0`;
} else if (bumpType === "minor") {
  newVersion = `${major}.${minor + 1}.0`;
} else {
  newVersion = `${major}.${minor}.${patch + 1}`;
}

console.log(`New version: ${newVersion}`);
console.log("");

// Ask for confirmation
if (process.argv.includes("--dry-run")) {
  console.log("Dry run - no changes made");
  process.exit(0);
}

// Execute the bump
console.log(`Running: bun run release:${bumpType}`);
try {
  execSync(`git add -A && bun exec standard-version --release-as ${bumpType}`, { stdio: "inherit" });
} catch (e) {
  console.error("Release failed:", e.message);
  process.exit(1);
}

// Fail hard if the version is out of sync across the three source-of-truth files.
// Run via `bun run check:version`. Wired into:
//   - the `release` npm script (preflight before standard-version)
//   - CI (build.yml, before `bun run tauri build` in every job)
// so a version mismatch is a hard error, not a silently-mislabeled release.
//
// Background: standard-version bumps package.json; Cargo.toml + tauri.conf.json
// must bump in lockstep (via .versionrc.js bumpFiles). If they ever drift
// (manual edit, a broken .versionrc, a re-added shadowing config), this guard
// turns it into a build failure.
import { readFileSync } from "fs";
import { resolve } from "path";

const scriptDir = new URL(".", import.meta.url).pathname;
const rootDir = resolve(scriptDir, "..");

function readJsonVersion(file: string): string {
	let json;
	try {
		json = JSON.parse(readFileSync(file, "utf8"));
	} catch {
		throw new Error(`Failed to parse ${file}`);
	}
	if (typeof json.version !== "string" || !json.version) {
		throw new Error(`No top-level "version" field in ${file}`);
	}
	return json.version;
}

function readCargoVersion(file: string): string {
	const contents = readFileSync(file, "utf8");
	const match = contents.match(/^version\s*=\s*"([^"]+)"/m);
	if (!match) {
		throw new Error(`No version line in ${file}`);
	}
	return match[1];
}

const pkgVersion = readJsonVersion(resolve(rootDir, "package.json"));
const tauriVersion = readJsonVersion(
	resolve(rootDir, "src-tauri", "tauri.conf.json"),
);
const cargoVersion = readCargoVersion(
	resolve(rootDir, "src-tauri", "Cargo.toml"),
);

console.log(`package.json    : ${pkgVersion}`);
console.log(`tauri.conf.json : ${tauriVersion}`);
console.log(`Cargo.toml      : ${cargoVersion}`);

const versions = new Set([pkgVersion, tauriVersion, cargoVersion]);
if (versions.size !== 1) {
	console.error(
		`\n❌ Version mismatch: package.json=${pkgVersion}, tauri.conf.json=${tauriVersion}, Cargo.toml=${cargoVersion}`,
	);
	console.error(
		`   Fix the stale file(s) so all three match, or cut the release with "bun run release"`,
	);
	console.error(
		`   (standard-version bumps all three together via .versionrc.js).`,
	);
	process.exit(1);
}

console.log(`\n✅ All version sources in sync: ${pkgVersion}`);

// standard-version config
// https://github.com/conventional-changelog/standard-version

import cargoUpdater from "./scripts/cargo-updater.js";

/** @type {import('standard-version').Configuration} */
export default {
	types: [
		{ type: "feat", section: "Features" },
		{ type: "fix", section: "Bug Fixes" },
		{ type: "perf", section: "Performance" },
		{ type: "refactor", section: "Refactoring" },
		{ type: "docs", section: "Documentation" },
		{ type: "test", section: "Tests" },
		{ type: "ci", section: "CI/CD" },
		{ type: "chore", section: "Maintenance", hidden: false },
	],
	bumpFiles: [
		{
			filename: "package.json",
			type: "json",
		},
		{
			filename: "src-tauri/Cargo.toml",
			type: "toml",
			updater: cargoUpdater,
		},
		{
			filename: "src-tauri/tauri.conf.json",
			type: "json",
		},
	],
	packageFiles: ["package.json"],
	bumpInChangelog: "package.json",
	tagPrefix: "companion/v",
	commitUrlFormat:
		"https://github.com/ManotLuijiu/amos-companion/commit/{{hash}}",
	compareUrlFormat:
		"https://github.com/ManotLuijiu/amos-companion/compare/{{previousTag}}...{{currentTag}}",
	issueUrlFormat: "https://github.com/ManotLiuJiu/amos-companion/issues/{{id}}",
};

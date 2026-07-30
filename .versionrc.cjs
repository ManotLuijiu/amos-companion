/** Versionrc configuration for standard-version
 * Ensures all version files are synced when bumping
 */
module.exports = {
	bumpFiles: [
		{ filename: "package.json", type: "json" },
		{
			filename: "src-tauri/Cargo.toml",
			type: "plain-text",
			updater: require("./scripts/cargo-version-updater.cjs"),
		},
		{ filename: "src-tauri/tauri.conf.json", type: "json" },
	],
	packageLock: "package-lock.json",
};

// .versionrc.js - standard-version configuration for AMOS Companion
// Handles: package.json, Cargo.toml, tauri.conf.json

const detectIndent = require("detect-indent");
const detectNewline = require("detect-newline");

const cargoUpdater = {
	readVersion(contents) {
		const match = contents.match(/version\s*=\s*"([^"]+)"/);
		return match ? match[1] : null;
	},
	writeVersion(contents, version) {
		return contents.replace(/version\s*=\s*"[^"]+"/, `version = "${version}"`);
	},
};

const tauriUpdater = {
	readVersion(contents) {
		try {
			const json = JSON.parse(contents);
			return json.package?.version || json.productName?.version;
		} catch {
			return null;
		}
	},
	writeVersion(contents, version) {
		try {
			const json = JSON.parse(contents);
			if (json.package) json.package.version = version;
			if (json.productName) json.productName.version = version;
			const indent = detectIndent(contents).indent || 2;
			const newline = detectNewline(contents) || "\n";
			return JSON.stringify(json, null, indent) + newline;
		} catch {
			return contents;
		}
	},
};

module.exports = {
	packageFiles: ["package.json"],
	bumpFiles: [
		"package.json",
		{ filename: "src-tauri/Cargo.toml", updater: cargoUpdater },
		{ filename: "src-tauri/tauri.conf.json", updater: tauriUpdater },
	],
	tagPrefix: "companion/v",
	skip: {
		changelog: true,
	},
};

// Cargo.toml updater for standard-version
// Updates version in: version = "x.y.z"

module.exports = {
	parse(version) {
		const match = version.match(/^version = "([^"]+)"$/);
		if (match) {
			return {
				version: match[1],
				rawVersion: version,
			};
		}
		return null;
	},
	stringify(version) {
		return `version = "${version}"`;
	},
};

/** Cargo.toml version updater for standard-version */
module.exports = {
  encode: (content, version) => content.replace(
      /^version = ".*?"/m,
      `version = "${version}"`
    ),
  decode: (content) => {
    const match = content.match(/^version = "(.*?)"/m);
    return match ? match[1] : null;
  }
};

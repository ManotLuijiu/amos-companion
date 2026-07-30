# Release Guide

## Version Files

This project has 3 version files that must be kept in sync:

| File | Format |
| ------ | -------- |
| `package.json` | `"version": "1.6.4"` |
| `src-tauri/Cargo.toml` | `version = "1.6.4"` |
| `src-tauri/tauri.conf.json` | `"version": "1.6.4"` |

## Release Steps

### 1. Update All Versions

Edit all 3 files to the new version:

```bash
# Edit package.json
sed -i 's/"version": "X.Y.Z"/"version": "1.6.5"/' package.json

# Edit Cargo.toml
sed -i 's/version = "X.Y.Z"/version = "1.6.5"/' src-tauri/Cargo.toml

# Edit tauri.conf.json
sed -i 's/"version": "X.Y.Z"/"version": "1.6.5"/' src-tauri/tauri.conf.json
```

Or manually edit these lines:

```diff
# package.json
- "version": "1.6.4",
+ "version": "1.6.5",

# src-tauri/Cargo.toml
- version = "1.6.4"
+ version = "1.6.5"

# src-tauri/tauri.conf.json
- "version": "1.6.4",
+ "version": "1.6.5",
```

### 2. Verify Versions Match

```bash
bun run check:version
```

Should output:

```
package.json    : 1.6.5
tauri.conf.json : 1.6.5
Cargo.toml      : 1.6.5

✅ All version sources in sync: 1.6.5
```

### 3. Commit

```bash
git add -A
git commit -m "release: 1.6.5"
```

### 4. Tag & Push

```bash
git tag companion/v1.6.5
git push
git push --tags
```

GitHub Actions will auto-build on the tag.

## Version Number Guidelines

| Type | Example | When |
| ------ | --------- | ------ |
| **Patch** | 1.6.4 → 1.6.5 | Bug fixes, small improvements |
| **Minor** | 1.6.5 → 1.7.0 | New features, backward compatible |
| **Major** | 1.6.5 → 2.0.0 | Breaking changes |

## Troubleshooting

### CI Fails: "No version line in Cargo.toml"

- Check that Cargo.toml has: `version = "X.Y.Z"` (no trailing comment)
- The version line must be exactly: `version = "X.Y.Z"`

### CI Fails: Version mismatch

- All 3 files must have exactly the same version number
- Run `bun run check:version` to verify

### Forgot to tag?

```bash
git tag companion/v1.6.5 HEAD
git push --tags
```

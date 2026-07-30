# Release Guide

## Quick Release

Use `release-it` to bump version + create Git tag + GitHub Release:

```bash
bun run release:patch   # 1.6.5 → 1.6.6
bun run release:minor   # 1.6.5 → 1.7.0
bun run release:major   # 1.6.5 → 2.0.0
```

This automatically:

1. Updates all 3 version files
2. Creates Git commit
3. Creates Git tag (`companion/vX.Y.Z`)
4. Creates GitHub Release

## Version Files

| File | Format |
| ------ | -------- |
| `package.json` | `"version": "X.Y.Z"` |
| `src-tauri/Cargo.toml` | `version = "X.Y.Z"` |
| `src-tauri/tauri.conf.json` | `"version": "X.Y.Z"` |

All synced automatically by `release-it`.

## Manual Version Bump

If release-it fails, manually update all 3 files:

```bash
# Edit these files to same version:
# package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json

bun run check:version  # Verify sync
```

## Troubleshooting

### CI Fails: "No version line in Cargo.toml"

- Check Cargo.toml has: `version = "X.Y.Z"` (no trailing comment on same line)
- The regex expects: `^version = "[^"]+"$`

### Forgot to push?

```bash
git push
git push --tags
```

## Version Number Guidelines

| Type | Example | When |
| ------ | --------- | ------ |
| **Patch** | 1.6.5 → 1.6.6 | Bug fixes |
| **Minor** | 1.6.6 → 1.7.0 | New features |
| **Major** | 1.6.6 → 2.0.0 | Breaking changes |

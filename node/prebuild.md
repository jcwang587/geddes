## Prebuild (Version Sync)

When preparing a release:

1. Update the `version` field in the `[package]` section of `Cargo.toml`.
2. Refresh `Cargo.lock` from repo root (this updates the `geddes` package version entry):

```bash
cargo check
```

3. Sync Node package version from repo root:

```bash
npm --prefix node run sync:version
```

Equivalent command from `node/`:

```bash
npm run sync:version
```

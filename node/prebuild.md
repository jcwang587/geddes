## Prebuild (Version Sync)

When preparing a release:

1. Update `[package].version` in `Cargo.toml`.
2. Sync Node package version from repo root:

```bash
npm --prefix node run sync:version
```

Equivalent command from `node/`:

```bash
npm run sync:version
```


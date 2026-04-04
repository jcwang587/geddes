## Prebuild (Version Sync)

The root `Cargo.toml` version is the single source of truth for the package version.

After bumping the root crate version:

- Rust uses the version directly from `Cargo.toml`
- Python derives its version from the Rust crate via `maturin`, so `pyproject.toml` does not need a separate version edit
- Node syncs the version into `node/package.json`, `node/package-lock.json`, and `node/Cargo.toml`
- The auto-generated `node/index.js` loader also embeds version checks and must be regenerated after the bump

For prereleases, note that Rust/Node use semver prerelease syntax such as `0.3.0-alpha.0`, while the built Python package will appear in PEP 440 form such as `0.3.0a0`.

## Recommended Release Flow

Run these commands from the repository root:

1. Update `[package].version` in `Cargo.toml`.
2. Sync the Node manifest versions from the root crate version:

```bash
npm --prefix node run sync:version
```

3. Refresh both Rust lockfiles so the package entries match the new version:

```bash
cargo check
cargo check --manifest-path node/Cargo.toml
```

4. Regenerate the NAPI-RS Node loader and related publish metadata so `node/index.js` picks up the new version checks:

```bash
npm --prefix node run build
```

For publish-time preparation, this is also acceptable:

```bash
npm --prefix node run prepublishOnly
```

## Notes

- `npm --prefix node run sync:version` updates manifest files only; it does not regenerate `node/index.js`.
- `cargo update` is not the preferred version-bump step here because it also updates dependency resolution. Use the `cargo check` commands above to refresh lockfiles for the current manifests.
- If you are already inside `node/`, the equivalent sync/build commands are `npm run sync:version` and `npm run build`.

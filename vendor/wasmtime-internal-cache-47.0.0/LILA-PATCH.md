# Lila published Wasmtime cache identity

Upstream is the crates.io `wasmtime-internal-cache` 47.0.0 archive, SHA-256
`1abeb39ab399a36987c51a0059f08dd8b2c5270d7b83edfcaf6003e5704e2db2`.
The complete Apache-2.0 WITH LLVM-exception license and upstream source remain
present. The original `.cargo_vcs_info.json` is retained exactly: upstream
commit `39d67168cdb6911ffdc961381314b6fed62e3c93`, path `crates/cache`.
`Cargo.toml.orig` records the unchanged upstream workspace manifest.

Upstream `build.rs` uses `git rev-parse HEAD` to select either the Git revision
and executable-mtime invalidation, or the package version. A published registry
crate beneath an unrelated Git repository therefore adopts that repository's
revision and fragments its native cache by executable mtime. Identical native
runtime modules then miss between Lila's CLI, workers and test executables.
There is no runtime `CacheConfig` override for this compile-time choice.

The patch checks for Cargo's `.cargo_vcs_info.json` packaging marker before
Git discovery. Published and vendored packaged source uses the upstream release
policy: package version without executable mtime. A source checkout without
that marker retains the original Git revision plus executable-mtime policy,
including invalidation for uncommitted development changes. Git lookup failure
retains the original package-version fallback. Git discovery now explicitly
starts at `CARGO_MANIFEST_DIR`, matching Cargo's build-script working directory.

[Cargo documents the marker as packaging metadata](https://doc.rust-lang.org/cargo/commands/cargo-package.html#cargo_vcs_infojson-format).
It identifies packaging form, not unchanged compiler provenance: Cargo describes
it as best effort, and packaged source can include uncommitted changes. This
patch changes only cache namespace discovery. Wasmtime's native compiler and
runtime remain the pinned, unmodified registry dependencies. Any future local
modification or vendoring of those native compiler/runtime components requires
a separate compiler-identity review; this marker alone cannot establish their
compatibility. Retain the marker when copying this package into `vendor/`.

No cache key hashing, serialization, compatibility checks, native code, cache
budget, retention policy or production deadline changes. The only modified
upstream files are `build.rs` and the normalized `Cargo.toml` (test registration).
`tests/compiler_identity.rs` adds packaged-ancestor, tracked-vendor,
development-checkout and unavailable-Git regressions using only the standard
library and Git. Lila's Engine process regression separately checks native R
reuse across identical executable bytes with different verified mtimes while
different P modules miss and execute to their own expected results.

The focused identity tests can run without resolving upstream test-only
packages, inside Lila's normal bounded verification owner:

```sh
CARGO_MANIFEST_DIR="$PWD/vendor/wasmtime-internal-cache-47.0.0" \
CARGO_PKG_VERSION=47.0.0 \
rustc --edition 2024 --test vendor/wasmtime-internal-cache-47.0.0/tests/compiler_identity.rs \
  -o target/verification-tmp/wasmtime-cache-compiler-identity
./target/verification-tmp/wasmtime-cache-compiler-identity --test-threads=1
cargo test --locked -p lila-engine --test runtime_artifact_cache \
  native_runtime_is_reused_by_executables_with_different_mtimes -- --exact --test-threads=1
```

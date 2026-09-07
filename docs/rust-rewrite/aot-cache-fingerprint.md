# Program-Wasm cache compiler identity

The program cache must miss whenever an input capable of changing compilation
changes. The v3 fingerprint covers the frontend, IR, AOT backend, runtime, Intl,
engine source/configuration, workspace manifests and lockfile, and patched
vendored source. It also covers build scripts and non-Rust embedded resources.
Cargo.lock alone does not identify local edits to path or patched dependencies.

Inputs are sorted and length-framed before hashing; checkout location and file
creation order do not change identity. Missing inputs and non-regular inputs
fail the build rather than silently retaining a stale identity. Generated
`target` directories and Git administrative directories are excluded.

Coverage is intentionally conservative: changes to files under a covered crate
may invalidate program-Wasm entries even when they are not semantic changes.
This does not invalidate the independently keyed Cranelift stencil cache and
does not claim a new Test262 result.

Validation:

```sh
cargo test --locked -p lila-engine --test compiler_fingerprint
```

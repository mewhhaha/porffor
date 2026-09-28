# Lila patch provenance

- Upstream package: `wasmtime-environ` 49.0.1 from crates.io.
- Registry checksum: `98f2ca21b23a51c1b1944297e9671a628c368e92aa6250e4c38f48e2dc251f0c`.
- The upstream `.cargo_vcs_info.json`, `Cargo.toml.orig`, `README.md`, and `LICENSE` are retained.
- Lila stores optional post-register-allocation stack metadata parallel to compiled function locations. This changes the serialized `CompiledFunctionsTable` layout.

The stack-budget metadata change is tied to Wasmtime 49.0.1 and must be reviewed together with the corresponding `cranelift-codegen`, `wasmtime-internal-cranelift`, and `wasmtime` patches. Persisted native artifacts require a cache-format version change.

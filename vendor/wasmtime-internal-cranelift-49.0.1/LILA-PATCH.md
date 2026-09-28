# Lila patch provenance

- Upstream package: `wasmtime-internal-cranelift` 49.0.1 from crates.io.
- Registry checksum: `b7d19868a30cc20df6f6648c9a4b3f700fcd1ddd0a63af9a98cb75f2b58dc874`.
- The upstream `.cargo_vcs_info.json`, `Cargo.toml.orig`, `README.md`, and `LICENSE` are retained.
- Lila captures each compiled function's conservative frame bound and active-frame SP-to-FP offset from Cranelift's finalized post-register-allocation metadata, then returns it with the function location for serialization by `wasmtime-environ`.

The stack-budget metadata change is tied to Wasmtime 49.0.1 and must be reviewed together with the corresponding `cranelift-codegen`, `wasmtime-environ`, and `wasmtime` patches.

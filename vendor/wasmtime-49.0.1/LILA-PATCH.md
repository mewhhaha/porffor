# Lila patch provenance

- Upstream package: `wasmtime` 49.0.1 from crates.io.
- Registry checksum: `942ddd2fc5800ec01d4b11c28ad62ab00517dcd416aff728f7d73fddf75872c1`.
- The upstream `.cargo_vcs_info.json`, `Cargo.toml.orig`, `README.md`, and `LICENSE` are retained.
- Lila exposes defined-function stack-layout metadata on `Module` and a `Caller::wasm_stack_budget` query. The query reconstructs the active guard wrapper's Wasm SP from Wasmtime's saved Wasm FP and that wrapper's Cranelift SP-to-FP offset, then measures remaining bytes to Wasmtime's soft Wasm stack limit.

The stack-budget metadata change is tied to Wasmtime 49.0.1 and must be reviewed together with the corresponding `cranelift-codegen`, `wasmtime-environ`, and `wasmtime-internal-cranelift` patches. Persisted native artifacts require a cache-format version change.

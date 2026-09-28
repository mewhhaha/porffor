# Lila patch provenance

- Upstream package: `cranelift-codegen` 0.136.1 from crates.io.
- Registry checksum: `e1af881f3b3392433e6f4f9b172b8d99270c538db4effea62b28bc48f02f0833`.
- The upstream `.cargo_vcs_info.json`, `Cargo.toml.orig`, `README.md`, and `LICENSE` are retained.
- Lila adds the post-register-allocation `FrameLayout::stack_size_bound` and carries it with `MachBufferFrameLayout`. The bound uses the same incoming, tail-call, setup, callee-save, fixed-storage, and outgoing-argument accounting as Cranelift's own frame-limit check.

The stack-budget metadata change is tied to Wasmtime 49.0.1 and must be reviewed together with the corresponding `wasmtime-environ`, `wasmtime-internal-cranelift`, and `wasmtime` patches.

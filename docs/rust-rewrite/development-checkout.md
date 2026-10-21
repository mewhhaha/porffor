# Development checkout inputs

Use the pinned toolchain in `rust-toolchain.toml`. A full checkout is the simplest
development input. In an existing cone-mode sparse checkout, retain these trees:

```sh
git sparse-checkout add .cargo .github crates scripts vendor test262 docs tasks snapshots
```

Cone mode also retains root files, including `Cargo.toml`, `Cargo.lock`,
`rust-toolchain.toml`, `AGENTS.md`, and `README.md`. This recipe deliberately keeps
whole crate and vendor trees: build scripts consume generated data, source
identities, and fixtures outside a selected crate's Rust modules. Do not infer a
smaller build input from the package you intend to test. Historical `tmp/` data
is not a compiler input; a historical replay may require its separately recorded
evidence inputs.

Keep `.cargo/config.toml`. Its linker flags participate in Cargo fingerprints;
omitting it can force a second build graph and expensive relinks. On Linux the
default requires `lld`; its comment documents the explicit override for hosts
without it. Keep the same flags when reusing a Cargo target directory. Do not
replace that directory with verification receipts or assume an artifact's name
proves its source identity.

Before building, inspect the selected graph without compiling:

```sh
cargo metadata --locked --offline --no-deps --format-version 1
python3 -B scripts/intl_source_identities.py
```

Offline metadata assumes the configured toolchain and dependencies are already
installed. For source-only tooling tests, use `PYTHONPATH=scripts`; individual
scripts executed from `scripts/` find their adjacent helpers normally. See the
[batch workflow](batch-workflow.md) for resource policy and verification commands.

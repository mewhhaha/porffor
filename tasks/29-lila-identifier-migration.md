# T29 — Migrate Rust identifiers from Porffor to Lila

**Status:** Complete — coordinated cutover and required verification are green as of 2026-08-13

**Parallel group:** Repository identity
**Depends on:** T28
**Blocks:** A fully Lila-native public API and distribution identity

This identity cleanup does not block Test262 work or the T26 conformance gate.

## Current repository state

The identity audit explicitly excludes only the status-artifact upgrader and
its regression fixture from legacy-token scanning. Those two tooling files
must spell the retired generated-block identity in order to recognize and test
historical inputs; the exception does not admit that identity on a product,
package, command, cache, host or publication surface. The upgrader regression
suite remains the authority for that narrow read-only migration boundary.

The public project is Lila and the legacy JavaScript implementation is gone.
The coordinated clean-break rename has moved all workspace crates, Rust
imports, the `lila` executable, environment/config/cache names, diagnostics,
generated helper namespaces and the Wasm host ABI. CI runs
`scripts/check-lila-identity.sh`, which rejects reintroduction of the retired
names outside the documented mapping/history/external-locator scopes.

Current snapshots and matrix caches use schema version 6 with the closed
`ArtifactProducer::Lila` identity. Version-4 and version-5 artifacts enter only
through a typed read-only decoder: they cannot be resumed, merged, rewritten or
published as current Lila evidence. The persisted-cache fixture covers the
Lila root, retired cache, sibling data and the global Wasmtime cache; its exact
integration test is green. The schema-v6 producer, version-4/version-5
read-only decoder, fresh-start journal behavior, CLI help/smoke, fake suites,
focused real-suite path, identity guard, workspace checks, and the complete
engine/CLI inventories are green in the final verification batch.

The pre-migration inventory is frozen against commit
`7ac4ee8a80e4e58b3dfb1adfece974f9f0a19e27` in
`docs/rust-rewrite/lila-identity-migration.md`. Its machine-readable mapping is
`docs/rust-rewrite/lila-identity-map.tsv`. The mapping includes the less-visible
`__porf*` generated JavaScript namespace, `$Porffor*` IR property keys,
`$porffor$module$*` linker names and `porf_host` Wasm import ABI.

## Objective

Design and execute one coordinated migration from transitional Porffor
identifiers to a coherent Lila Rust library, CLI, compiler, cache, diagnostics,
and artifact identity. Encode compatibility decisions explicitly instead of
leaving mixed names indefinitely.

## Work items

1. **Complete.** Inventory every public and persisted identifier: Cargo packages and crate
   imports, binary names, library exports, environment variables, cache paths
   and keys, host ABI symbols, snapshot schemas, diagnostics, docs, scripts,
   workflows, and release artifacts.
2. **Complete.** Select collision-free canonical Lila names and document the mapping before
   code changes begin.
3. **Complete.** Decide which public or persisted surfaces require a bounded migration path.
   Before 1.0, prefer clean breaks; retain an alias only when it prevents real
   user data loss or enables a deliberately staged distribution transition.
   The only bounded compatibility path is a typed read-only decoder for
   Porffor-era Test262 snapshots; it cannot resume, merge or publish them and
   expires after the first verified full pinned Lila aggregate, no later than
   1.0. External repository/DNS locators are retained resources, not aliases.
4. **Complete.** Rename foundation crates and shared types first, then consumers, CLI and
   scripts, persisted data, documentation, and automation in dependency order.
5. **Complete.** Add exact boundary checks that reject newly introduced transitional names
   while allowing only explicitly documented temporary aliases.
6. **Complete.** The mapped version-6
   Lila producer contract and typed read-only decoder for version-4/version-5
   Test262 snapshots are implemented and verified, with no unowned
   transitional identifiers in current product surfaces.

## Out of scope

- Restoring or renaming the retired JavaScript implementation.
- Changing ECMAScript behavior or conformance accounting.
- Selecting a package registry, release cadence, or domain migration unless
  those decisions are required for the identifier contract.

## Acceptance criteria

- A checked-in mapping accounts for every discovered identifier and persisted
  namespace before implementation begins.
- Workspace crates, Rust imports, CLI identity, environment variables, caches,
  diagnostics, host interfaces, status artifacts, docs, and CI use the selected
  canonical names consistently.
- Any compatibility alias has an owner, reason, warning behavior, and removal
  deadline; otherwise the migration is a clean break.
- Cache migration cannot delete unrelated data and handles existing entries
  deterministically.
- Product, fake-suite, focused real-Test262, and repository-contract checks pass
  after the rename.
- Repository-wide searches find no unexplained transitional identifier.

## Required tests

The exact commands depend on the approved name mapping, but the implementation
must include:

```sh
cargo fmt --all -- --check
cargo xc
./scripts/check-task-plan.sh
./scripts/check-no-interpreter-in-product-graph.sh
```

Also run the renamed CLI's help, compile/run a representative JavaScript
fixture through Wasm-AOT, execute both fake Test262 suites, verify persisted
cache migration with old and new entries, and run the identifier-boundary audit.

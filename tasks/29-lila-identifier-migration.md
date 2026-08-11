# T29 — Migrate Rust identifiers from Porffor to Lila

**Status:** Open

**Parallel group:** Repository identity
**Depends on:** T28
**Blocks:** A fully Lila-native public API and distribution identity

This identity cleanup does not block Test262 work or the T26 conformance gate.

## Current repository state

The public project is Lila and the legacy JavaScript implementation is gone,
but the Rust workspace deliberately retains transitional identifiers:
`porffor-*` crate and package names, the `porf` executable, `PORFFOR_*`
environment variables, cache namespaces, diagnostics, internal ABI symbols,
and Test262 artifact fields. They form a connected compatibility surface and
must not be renamed piecemeal.

## Objective

Design and execute one coordinated migration from transitional Porffor
identifiers to a coherent Lila Rust library, CLI, compiler, cache, diagnostics,
and artifact identity. Encode compatibility decisions explicitly instead of
leaving mixed names indefinitely.

## Work items

1. Inventory every public and persisted identifier: Cargo packages and crate
   imports, binary names, library exports, environment variables, cache paths
   and keys, host ABI symbols, snapshot schemas, diagnostics, docs, scripts,
   workflows, and release artifacts.
2. Select collision-free canonical Lila names and document the mapping before
   code changes begin.
3. Decide which public or persisted surfaces require a bounded migration path.
   Before 1.0, prefer clean breaks; retain an alias only when it prevents real
   user data loss or enables a deliberately staged distribution transition.
4. Rename foundation crates and shared types first, then consumers, CLI and
   scripts, persisted data, documentation, and automation in dependency order.
5. Add exact boundary checks that reject newly introduced transitional names
   while allowing only explicitly documented temporary aliases.
6. Remove each temporary alias on its recorded deadline and finish with no
   unowned Porffor identifiers in current product surfaces.

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

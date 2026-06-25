# Test262 Shortcut Inventory

This is the initial checked-in inventory for T03. Regenerate the mechanical
line-level report with:

```sh
./scripts/audit-test262-shortcuts.sh
```

Check that the current tree does not exceed the documented ceiling in
`shortcut-allowlist.tsv` with:

```sh
./scripts/audit-test262-shortcuts.sh --check
```

The current scanner intentionally over-includes test262 runner code. Each match
must be classified before T03 closes as one of:

- legitimate harness adaptation;
- temporary diagnostic instrumentation;
- semantic shortcut to remove under the owning task.

## Current Mechanical Buckets

| Bucket | Count | Primary owner | Removal/accounting rule |
|---|---:|---|---|
| Path-based rewrite entrypoints | 107 | T03 inventory plus owning feature lane | Remove as the corresponding builtin/runtime semantics land; owner follows the Test262 prefix map in `test262/backlog/ownership-map.tsv`. |
| Direct path predicates | 358 | T03/T01 plus owning feature lane | Keep snapshot routing and matrix selection only when they do not change JavaScript semantics; semantic path branches need a removal task. |
| Source-text predicates | 590 | T03/T04/T07/T13 plus owning feature lane | Parser, dynamic-source and unsupported-feature detection must become structured metadata or IR diagnostics instead of source text matching. |
| Harness and helper reductions | 337 | T03 | Keep only documented shell adaptation; helper reductions that implement product semantics must move into compiler/runtime code or remain visible debt. |

## Known High-Risk Areas

| Area | Evidence pattern | Owner |
|---|---|---|
| Static Test262 rewrites in `materialize_test` | `rewrite_*(&case.path)` dispatch chain | Owning feature task by prefix; T03 tracks inventory |
| Source-based feature support checks | `case.original_source.contains(...)` and `source.contains(...)` | T04/T07/T13 depending on boundary |
| Reduced harness helpers | `assert.sameValue = function`, `assert.throws = function`, `prelude.contents` checks | T03 |
| TypedArray helper skipping | `wasm_aot_rewrite_skips_test_typed_array` | T17 |
| Cross-realm/static host materializations | `$262.createRealm` source rewrites | T06/T13/T24 |

## Next T03 Steps

1. Replace bucket-level allowlist ceilings with line-level classifications as
   branches are retired or ownership is clarified.
2. Add a CI check that fails when a new exact-path or source-text semantic
   branch is added without an allowlist entry.
3. Retire path/source branches as feature tasks replace them with general
   semantics.

# Symbol heap-slot identity authority

Current dry source — 2026-10-05: the raw-offset source-mirror target below
is retired from the atomic semantic GC draft. It checked passive layout
spelling, offsets and source occurrence counts. The current GC authoring uses
typed registry fields and complete semantic value owners; these earlier
mirrors cannot verify that representation or its behavior. Historical source
descriptions, commands and results below retain their original scope.

The full GC source cutover, remaining Temporal and AsyncDisposableStack
authoring, compiler checks and runtime verification remain pending. See the
[current value/heap architecture](../value-heap-gc.md). Weak reachability retains
its explicit [unavailable facility boundary](weak-unavailable-runtime-boundary.md).
No verification or conformance result is inferred from this retirement.

## Closed layout identities

The passive Symbol record layout contains exactly four capability-free
`SymbolHeapSlot` identities in description-tag, description-payload,
registry-key-payload and symbol-id order:

- `DescriptionTag`;
- `DescriptionPayload`;
- `RegistryKeyPayload`;
- `SymbolId`.

One private exhaustive `metadata()` projection is the sole authority for all
four identities' record names, slot names, offsets, widths and pointer
classifications. The description tag remains a scalar 8-byte word at offset
zero. The description and registry-key payloads remain pointer-classified
8-byte words at offsets 8 and 16. The symbol identity remains scalar at offset
24. An arbitrary row cannot trace the tag or identity, or omit either payload
from the pointer census.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form Symbol record rows. The
bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change Symbol allocation, emitted Wasm, description or registry semantics, or
collector execution.

```sh
cargo test -p lila-aot-wasm --test symbol_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::symbol_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_symbol_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/symbol_heap_slot_structure.rs
git diff --check
```

The recursive structure target passes `4/4`, the exact heap owner witness
passes `1/1`, and the collision/pointer registry filter passes `2/2`. The
shared `cargo xc` checkpoint is green. This is a passive layout migration, so
no semantic golden or Test262 rerun was performed.

# Symbol receiver operation ownership

The four receiver operations on Symbol prototypes share one physical
receiver algorithm. `SymbolReceiverOperation` is the private, non-capability
authority that selects the exact diagnostic for `description`, `toString`,
`valueOf` and `[Symbol.toPrimitive]`. Each public dispatcher constructs one
named operation and moves it into `emit_this_symbol_value_to_local`; the sole
consuming projection selects the message before the emitted receiver walk.
That is the sole consuming projection for this authority.

`[Symbol.toPrimitive]` previously duplicated that receiver walk. It now uses
the same typed boundary and only performs its distinct result publication after
the helper has produced a Symbol payload. A representation or boxed-Symbol
change therefore has one owner, while adding a fifth operation requires an
exhaustive diagnostic decision before the crate builds.

The Rust-lexical structure guard pins the attribute-free four-row declaration,
seven production mentions, absence of clone/copy/debug/equality capabilities,
the four exact message rows, one typed consumer and exactly four producers. It
also ensures no dispatcher retains a raw receiver diagnostic or a second copy
of the algorithm.

Focused verification:

```sh
cargo test -p lila-aot-wasm --test symbol_receiver_operation_domain_structure -- --test-threads=1
```

The exact non-object and boxed-Symbol `[Symbol.toPrimitive]` leaves pass all
`4/4` sloppy/strict Wasm-AOT executions. Every failure bucket is zero and all
four outcomes are `Success`. The full Symbol tree remains deferred.
Test262 remains deferred beyond these exact witnesses. This contract does not close
Symbol identity, property-key ordering or T21's unavailable weak-reachability
backend.

## Batch AS dispatcher boundary

The seven-entry outer family now uses a private `SymbolBuiltin` with no derived
capabilities, and the raw emitter is private to `symbol.rs`. Standard dispatch
can reach it only through seven fixed Symbol entries and can neither construct
nor pass the raw family authority. The frozen 323-line domain/emitter selection
has SHA-256
`3296276e16255ea9aaf39f05b54b77414320a0f71d5c0d4c1a61ed04c1cef9b2`.
Restoring only the former derive and visibility reproduces that source exactly.
At the 2026-08-28 Batch AS checkpoint, `cargo xc` is green, the strengthened
receiver-operation structure target passes `4/4`, and the exact non-object and
boxed-Symbol `[Symbol.toPrimitive]` leaves pass all `4/4` sloppy/strict
Wasm-AOT executions with every failure bucket at zero. This source-equivalent
boundary claims no new Symbol behavior, broader conformance or published
conformance-count change.

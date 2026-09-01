# Temporal.Instant validated epoch proof

The two implemented epoch constructors must validate their computed BigInt
before `CreateTemporalInstant`. `UnvalidatedEpochNanoseconds` remains the named
pair used by producers, while private `EpochNanoseconds` is the proof returned
only after `emit_temporal_instant_validate_range` has emitted its range check.

`EpochNanoseconds` is non-`Copy` and derives no incidental capabilities.
`emit_alloc_validated_temporal_instant` takes the proof by value and immediately
destructures both fields before allocation. A second observation of the proof
therefore fails to compile, and adding a field to the unvalidated pair requires
the allocation boundary to decide how that field participates.

The Rust-lexical structure regression ignores nested comments and all Rust
string-literal forms before enforcing the 5 source-wide `EpochNanoseconds`
mentions: declaration, constructor result, construction, consumer parameter
and final destructuring. It also pins range-check-before-construction and both
builtin validate-before-allocate paths. This source-equivalent ownership
hardening changes neither the emitted validation nor the Temporal allocation.

Focused verification on 2026-08-27:

```sh
cargo test -p lila-aot-wasm --test temporal_instant_epoch_proof_structure
```

The structure target passes all 5 tests after compiling the backend. No broad
Cargo or Test262 suite was run for this invariant-only change.

# Generator delegate property capability

Status: implemented as a source-equivalent Wasm-AOT invariant boundary.

`GeneratorDelegateProperty` is the private seven-row authority for every
property read performed by synchronous and asynchronous generator delegation.
Its sole borrowed projection maps each row to the private
`GeneratorDelegatePropertyKey::{WellKnownSymbol, OrdinaryString}` domain. Both
domains derive no cloning, copying, debugging, equality or default capability.

The reader owns one property authority and exhaustively consumes the projected
key. The two Symbol rows retain the typed Symbol property-read path; the five
ordinary rows retain the string-pool lookup, temporary-local lifecycle and
abrupt-completion propagation. Fourteen exact producers remain in their
existing sync/async order.

`generator_delegate_property_domain_structure.rs` uses a dependency-free Rust
lexical scanner to exclude comments and literals and canonicalize raw
identifiers. It pins the recursive 24/11 identifier censuses, all seven exact
mappings, all fourteen ordered calls with their local forwarding, the sole key
route and both complete reader arms. The neighboring async-delegation guard
continues to own its separate two-row policy.

The property structure target passes `3/3`, the neighboring async-delegation
target passes `4/4`, and the five retained async-generator delegation CLI
witnesses pass `5/5`. No Test262 cohort, semantic golden or broad suite was run
for this capability-only follow-up.

This hardening changes only Rust capabilities and borrowing. It adds no emitted
Wasm instruction, generator behavior, conformance support or broad T15 claim.
Independent dry review is clean. The following shared workspace checkpoint
passes `cargo fmt --all -- --check`, `cargo xc`, the recursive module-boundary
check, the task-plan check and `git diff --check`.

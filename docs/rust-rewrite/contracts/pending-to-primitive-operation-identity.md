# Pending ToPrimitive operation identity

## Type-owned identity

`PendingToPrimitiveCompletion` can represent only a pending `ToPrimitive`
completion. Its private state contains exactly the result payload and tag
locals; it does not store a freely selected `MayThrowOperation`. Every raw
ToPrimitive producer constructs the same two-local token, and its routed
consumer reaches the private ToPrimitive finisher directly.

The other five consuming continuations need only the two result locals and no
longer discard a redundant operation field. Constructing a pending token for
GetV, ToLength, ToNumber or another operation is unrepresentable rather than a
debug assertion that disappears from release builds.

The later ordinary-receiver audit deletes the unreachable Function-only
producer pair. The live tagged path still constructs the Function receiver
choice directly, while the ordinary Object wrapper retains the other entry;
the raw pending-token producer census is therefore three rather than four.

## Capability boundary

The later ownership audit deletes `MayThrowOperation` entirely. Its value was
ignored by all three finishers, so passing `TO_LENGTH` to the ToPrimitive
finisher or `TO_NUMBER` to the ToLength finisher still compiled. The named
operation boundaries now own identity: the GetV wrapper selects
`SpecOperationIr::GetV`, while the ToNumber, ToLength and ToPrimitive wrappers
can reach only their corresponding private finisher. The pending completion
remains non-cloneable and moves to its exhaustive routing boundary.

This makes the false generic-marker states unrepresentable. The later route
audit also removes the two-variant `AbruptRoute`: GetV and builtin ToNumber each
own their fixed continuation directly, while `ToPrimitiveAbruptRoute` and
`ToLengthAbruptRoute` remain the behavioral authorities for sites that
genuinely select among multiple policies.

This changes only emitter-time Rust ownership. It does not change an operation
descriptor, completion route, emitted instruction or Wasm ABI slot.

```sh
cargo test -p lila-aot-wasm --test pending_to_primitive_operation_identity_structure
cargo test -p lila-aot-wasm --test may_throw_abrupt_route_ownership_structure
cargo test -p lila-aot-wasm --test conversion_error_realm_source_structure
cargo xc
git diff --check
```

The focused identity target passes `3/3`. The neighboring may-throw ownership
and conversion-Realm targets remain green at `4/4` each; the adjacent
conversion-capability target passes `2/2`. The shared `cargo xc` and workspace
hygiene gates are green with only existing warnings. The exact ToLength owner
and Error ToPrimitive/ToString CLI witnesses each pass `1/1`.

## Nonclaims

This closure does not migrate another abstract operation, add a completion
route, change error-Realm selection or complete the shared-operation catalog.

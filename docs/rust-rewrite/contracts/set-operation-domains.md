# Set operation domains

Status: implemented and focused-verified for the seven Wasm-AOT Set composition
and predicate methods, 2026-08-26.

## Closed domains

`SetPredicateOperation` owns the three public predicates, and
`SetAlgebraOperation` owns the four public composition methods. Neither domain
supports equality, so no new operation can inherit policy from an `if` default.

The iteration helpers accept only the operations legal for their traversal:

- receiver predicate iteration accepts `IsDisjointFrom | IsSubsetOf`;
- other predicate iteration accepts `IsDisjointFrom | IsSupersetOf`; and
- receiver algebra iteration accepts `Difference | Intersection`.

The public predicate dispatcher constructs those restricted operations through
one exhaustive match. The algebra operation exhaustively selects whether it may
iterate the receiver, while the other-iteration helper retains the complete
four-operation domain because every algebra method may use that path. Result
copying is also exhaustive: intersection starts empty; difference copies before
its size-directed traversal; symmetric difference and union copy after calling
`keys` and reading the returned iterator's `next` method, before calling `next`.

The has-method path for difference visits its private result copy. Mutating the
original receiver during `other.has` does not replace the original keys still
to be visited. Intersection retains its live receiver traversal, including
reinserted keys. The restricted operation selects those two sources explicitly.

Adding a public operation therefore requires selecting its initialization and
iteration plan. Adding a restricted helper operation requires implementing its
exact failure polarity or result mutation. There are no debug-only restrictions,
equality projections or unreachable invalid-operation arms in this boundary.

## Durable regression

`set_operation_domains_structure.rs` owns the five exact variant sets, seven
public producers, restricted helper signatures, exhaustive projections, helper
call census and the absence of equality/default escape hatches.

The finite CLI fixture selects both size-directed paths for difference,
intersection and disjointness. It also covers symmetric difference, union,
subset and superset with successful and failing predicate results, so every
restricted helper variant has an observable witness.

```sh
cargo test -p lila-aot-wasm --test set_operation_domains_structure --quiet
cargo test -p lila-cli --test cli iterator::run_wasm_backend_preserves_set_operation_domains -- --exact --test-threads=1
```

The shared semantic golden passes `2/2` in 722.99 seconds with 678 dumps. It
adds this witness plus the independent Array.fromAsync callback-Realm, Object
policy and Promise-mode witnesses, removes none and leaves all 674 retained
dumps equal after accounting normalization. Broad Test262 verification remains
deferred.

This type closure preserves valid Set values, ordering, size selection,
set-like observation and iterator closing. It does not claim the complete
pinned Set tree or weak-collection semantics.

`aot_set_algebra_copy_phases` covers receiver mutation in `has`, `keys`, the
iterator's `next` getter and its first `next` call. It retains intersection's
live traversal, duplicate suppression, and SameValueZero normalization. The
pinned SpiderMonkey difference, symmetric-difference and union cases supply
the corresponding complete harness consumers.

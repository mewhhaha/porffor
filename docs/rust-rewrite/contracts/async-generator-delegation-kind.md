# Async-generator delegation kind

Status: implemented and structurally verified on 2026-08-27.

## Boundary

`AsyncGeneratorDelegationKind::{YieldStar, ForAwaitYield}` is the closed
compile-time policy shared by the async-generator delegation emitter. There are
exactly two producers: delegated `yield*` selects `YieldStar`, while the
transparent-yield optimization for an async-generator `for await` loop selects
`ForAwaitYield`.

The shared emitter uses eight exhaustive matches to own every semantic
difference. Those matches select pending Throw and Return forwarding after
await settlement, delegate `throw` lookup, Return-or-Throw close eligibility,
missing-return completion, close-call arguments and the ordinary `next`
argument. No equality, wildcard or implicit default projects this policy.

The domain derives no cloning or copying capability. All eight independent
projections borrow the same compile-time policy while emitting one function,
so no projection can duplicate or consume the authority. It also derives no
debugging, equality or default capability and has no manual implementation.

## Durable evidence

`async_generator_delegation_kind_structure.rs` pins the exact attribute-free
two-row domain, exactly two producers, all eight borrowed exhaustive matches
and the established variant polarity at each projection. The existing
async-generator `yield*` wrapper fixtures and the pinned
`yield-promise-reject-next` Test262 leaves for both `yield*` and `for await`
remain the focused behavioral witnesses.

## Verification

Run the bounded structure target with:

```console
cargo test -p lila-aot-wasm --test async_generator_delegation_kind_structure
```

The structure target passes `4/4`, and the five exact async-generator
delegation CLI witnesses pass `5/5`. The six exact `yield*` and `for await`
Test262 leaves pass all `12/12` sloppy/strict Wasm-AOT executions with every
failure bucket at zero. This migration changes only Rust's compile-time policy
selection, so emitted Wasm is expected to remain byte-identical. The semantic
golden remains deferred. Independent review found and closed a guard gap in the
first three projection bodies; final re-review is clean. The shared checkpoint
passes `cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the
module-boundary check and the task-plan check.

## Nonclaims

No generator or iterator behavior change is claimed. This contract does not
expand supported delegation forms, `for await` assignment heads, iterator
closing, async-generator suspension or broader Test262 conformance.

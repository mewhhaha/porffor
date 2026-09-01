# `Array.prototype.flat` direct-entry and dispatch ownership

Status: dispatch invariant implemented and dry-reviewed for the Wasm-AOT
compiler on 2026-08-28. Focused runtime and Test262 verification remain at the
shared checkpoint.

## Single direct entry

Static direct `flat()` syntax delegates through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeFlat`. The shared direct-call boundary
evaluates the receiver once, propagates its abrupt completion, evaluates and
expands the complete argument list from left to right, and only then enters the
standard builtin.

The deleted `emit_array_flat_method_call` was a second entry owner. It compiled
each argument as an ordinary expression and then built an argv from those
locals. A `SpreadArgument` therefore reached the expression compiler outside a
call and was rejected instead of invoking the iterator protocol. The wrapper
also lacked the shared boundary's explicit abrupt-completion checks between the
receiver and argument expressions. With that owner absent, a stale direct call
fails to compile and every Flat call uses the complete argument-list boundary.

The canonical builtin's metadata length is zero, but that does not constrain
runtime argc. The compiler distinguishes zero arguments from a present first
argument, reads only argument zero as `depth`, and ignores later values after
the call boundary has evaluated them.

## Closed static dispatch

The `ExprIr::CallMethod` lowering in `functions.rs` is only a dispatch
optimization. Its static `flat()` seam locally owns the capability-free
`FlatMethodDispatch::{ArrayCanonical, GenericGetCall}` authority. A data
property whose function-target set contains exactly the one
`StandardBuiltinId::ArrayPrototypeFlat` target constructs `ArrayCanonical`.
Every absent, accessor, ambiguous or unknown target constructs
`GenericGetCall`.

One exhaustive match owns the sole canonical direct call and ordinary property
Get/Call fallthrough. The authority has no derives, wildcard, default,
kind-only shortcut or Array heap-shape shortcut. An own method, accessor or
Proxy receiver therefore cannot be bypassed by the canonical algorithm.

## Preserved Flat algorithm

`compile_array_prototype_flat_builtin` remains the sole standard entry and sole
Flat algorithm owner. Its body does not change in this closure. It still owns:

1. receiver conversion and one source-length snapshot;
2. default depth and first-argument numeric conversion;
3. Array species construction and result publication;
4. sparse `HasProperty` before indexed `Get`;
5. recursive flattening and positive-infinity depth; and
6. target-write failures and observable Proxy operations.

The canonical Flat compiler remains pinned at
`c83ffc356528d69e9de4a63e29cb30a4d55d751c662c30810aab2eba9c390c56`. The
earlier custom Array named-property path remains ahead of the direct builtin
branch. The neighboring `flatMap` Array/Iterator dispatch is unchanged.

## Durable evidence

`crates/lila-aot-wasm/tests/array_flat_algorithm_owner_structure.rs`
recursively pins:

- the exact capability-free two-state dispatch, one producer and one exhaustive
  consumer per state, singleton target proof and generic Get/Call fallthrough;
- the direct standard builtin selection, label and complete `args` forwarding
  only in the proven canonical state;
- absence of the deleted direct owner;
- one canonical compiler and one standard dispatcher consumer;
- receiver-before-arguments-before-call ordering in the shared direct-call
  boundary;
- zero-argc/default-depth versus sole argument-zero projection; and
- depth conversion, source-length observation, `HasProperty` and indexed `Get`
  order in the unchanged canonical compiler.

The focused CLI fixture `wasm_array_flat_argument_evaluation.js` uses an
accessor-backed source element. Its depth argument, ignored second argument and
trailing custom iterable spread record complete left-to-right evaluation before
the indexed getter records the start of flattening. The existing core and Proxy
access-count fixtures remain controls for the unchanged Flat algorithm.

`wasm_array_flat_own_method_dispatch.js` installs an own `flat` method on an
Array, observes its receiver and ordinary/spread arguments, returns a custom
result and requires the nested source to remain unchanged. It fails if a kind
or heap-shape heuristic restores unconditional canonical routing.

No neighboring structure guard used the deleted wrapper as a boundary, so this
closure changes no marker file.

## Verification

The five-test recursive owner target, exact own-method CLI control and existing
argument-evaluation, core and Proxy access-count controls are centralized
verification obligations. The canonical compiler remains pinned at the hash
above, and the removed source owner has zero Rust source matches.

Pinned Test262 controls are `non-numeric-depth-should-not-throw.js`,
`proxy-access-count.js`, `positive-infinity.js` and the call-expression
`spread-mult-iter.js`.

At the shared checkpoint, the recursive structure target passes `5/5`; the
exact own-method, argument-evaluation, core and Proxy CLI controls each pass
`1/1`; and the four pinned leaves each pass `2/2`, for `8/8` Wasm-AOT
executions with every failure bucket at zero. The shared `cargo xc` checkpoint
is green.

## Nonclaims

This closure changes only statically named calls whose actual `flat` property
is not proven to be the intrinsic. It does not change the canonical Flat
algorithm, remove a Test262 materializer, change a published conformance count
or claim the Array subtree green. It does not change `flatMap` or canonicalize
`concat`, `push` or another method.

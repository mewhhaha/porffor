# BigInt helper-operation runtime-code authority

Status: implemented as a source-equivalent Wasm-AOT serialization invariant.

## Closed operation

`BigIntHelperOp` is the narrow crate-visible authority for the fourteen legal
operations accepted by parameter 4 of the shared BigInt runtime helper. It
derives no cloning, copying, formatting, equality or default capability.

Arithmetic and bitwise source operators enter through their existing exhaustive
projections. Complement, BigInt comparison, mixed BigInt/Number comparison and
unary negation retain their direct named producers.

## Stable runtime codes

One borrowed exhaustive `runtime_code` projection owns the stable words 0
through 13. All twenty-one serialization sites use that projection: two carry
an operation parameter to the helper call and nineteen emit direct comparison
or normalization constants while compiling the helper body.

The projection is a compiler-time decision. Replacing the former casts changes
no emitted instruction, local, helper signature, operand order or numeric word.
A future Rust operation row must now choose its helper-ABI word before the
backend builds.

## Runtime decoder boundary

The generated helper still receives an `i64` and dispatches it through its
existing Wasm comparison tree. That tree retains its current XOR and remainder
fallthroughs. This contract closes typed Rust-to-Wasm serialization; it does not
claim exhaustive decoding of arbitrary runtime words or make invalid helper
arguments safe.

## Durable evidence

`bigint_helper_op_structure.rs` uses a dependency-free Rust lexical scanner to
exclude nested comments and every string/character literal form from its
recursive identifier and route censuses. It pins the exact fourteen-row
declaration and runtime-code/arithmetic/bitwise tables. The nine semantic
producers are bound to complete normalized calls, including their operand and
result-local order. Recursive lexical checks reject raw casts and UFCS or
associated-route alternatives, while exact helper-body decision regions bind
all nineteen direct projections to their `I64Const` semantics; the two dynamic
projections remain pinned to their complete helper-call instruction sequences.

The structure target passes `4/4`. The focused BigInt bitwise, exponentiation
and mixed Number relational CLI fixtures pass `3/3`, exercising
bitwise/shift/complement, exponentiation and comparison families. Six selected
arithmetic, unary-minus and BigInt relational Test262 leaves pass all `12/12`
sloppy/strict Wasm-AOT executions with every failure bucket at zero.

## Scope

This boundary adds no BigInt operation, representation, resource policy or
conformance claim. It does not complete T20 or change README status.
Independent dry re-review is clean after the producer and serializer guards
were hardened. The following shared workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, the recursive module-boundary check,
the task-plan check and `git diff --check`.

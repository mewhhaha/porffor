# TypedArray quantifier-kind authority

Status: implemented as a source-equivalent Wasm-AOT compile-time boundary.

## Closed authority

`TypedArrayQuantifierKind::{Every, Some}` is private to the Array builtin
module and derives no clone, copy, equality, debug or default capability. The
standard builtin dispatcher cannot construct or pass this authority. It calls
the exact public `every` or `some` compiler entry, and those two entries are the
only producers of the private kind.

The shared compiler borrows the authority in seven direct exhaustive matches:

- two missing-receiver compiler diagnostics;
- the incompatible-receiver TypeError;
- the non-callable-callback TypeError;
- callback-result polarity;
- the short-circuit Boolean result; and
- the terminal Boolean result.

There is no equality, Boolean, wildcard, default or unreachable projection.
Adding another quantifier therefore fails to compile until all seven semantic
decisions are explicit. Moving a new caller outside the Array builtin module
also cannot bypass the two named entry points by selecting a kind directly.

## Preserved behavior

This boundary changes only Rust policy ownership and observation. The two
entry points emit no Wasm themselves; the shared compiler retains the same
instructions, messages, locals, buffer witness, callback call, short-circuit
behavior and release order. `Every` still stops on the first falsy callback
result and defaults to `true`; `Some` still stops on the first truthy result
and defaults to `false`.

## Durable evidence

`typed_array_quantifier_family_witness_structure.rs` pins the private
capability-free declaration, the two wrapper producers, both standard dispatch
mappings, the private shared consumer and all seven borrowed exhaustive
projections. Its existing assertions continue to own validated TypedArray
entry, immutable buffer-witness use, callback ordering, polarity and the exact
local-release sequence.

The existing `wasm_typedarray_every_some.js` CLI fixture distinguishes both
quantifiers, callback polarity and method-entry behavior. No fixture,
Test262 materializer or published conformance count is changed by this
source-equivalent invariant.

The bounded structure target passes `4/4`, and the exact existing CLI fixture
passes `1/1`:

```console
cargo test -p lila-aot-wasm --test typed_array_quantifier_family_witness_structure -- --test-threads=1
cargo test -p lila-cli --test cli typed_array::run_wasm_backend_succeeds_for_typedarray_every_some_fixture -- --exact --test-threads=1
```

Both focused builds retain the working tree's existing warnings.

## Scope

This does not change generic Array `every` or `some`, TypedArray buffer
semantics, callback invocation, Realm selection or broader T16/T17
conformance. It does not claim the full Array or TypedArray trees are green.

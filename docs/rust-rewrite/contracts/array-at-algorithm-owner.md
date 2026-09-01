# TypedArray `at` direct-entry ownership

Status: implemented for the Wasm-AOT compiler on 2026-08-28. Focused and
shared-checkpoint verification is recorded below.

## Single direct entry

Static direct `at()` lowering retains its existing branch and strict TypedArray
receiver policy, but now delegates through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::TypedArrayPrototypeAt`. The shared boundary evaluates the
receiver once, propagates abrupt completion, evaluates and expands the complete
argument list from left to right, and only then enters the standard builtin.

The deleted `emit_array_at_method_call` compiled only `args.first()`. Later
argument expressions were never evaluated, and a later spread never invoked
its iterator protocol. With that owner absent, a stale direct call fails to
compile and the source argument list cannot be projected to one optional
expression before reaching the call boundary.

The canonical compiler reads only argument zero as the relative index. Later
values are evaluated by the call boundary but do not affect `at` semantics.

## Preserved receiver policy and algorithm

This direct route intentionally selects `TypedArrayPrototypeAt`, not
`ArrayPrototypeAt`. The standard dispatcher therefore constructs
`ArrayAtReceiverPolicy::TypedArray`, preserving validation and the
validated-method-entry buffer witness. The earlier string and custom Array
branches, the static `at` branch's receiver classification, and generic
`Array.prototype.at.call` remain unchanged.

`compile_array_prototype_at_builtin` and `emit_array_at_from_locals` do not
change in this closure. They remain the sole compiler and shared algorithm
owners for both generic Array-like and strict TypedArray entries. The shared
algorithm still owns receiver handling, length observation, strict TypedArray
validation, index coercion, relative-index bounds and the final indexed read.

## Durable evidence

`array_at_algorithm_owner_structure.rs` recursively pins:

- the exact strict TypedArray builtin selection, label and complete `args`
  forwarding;
- absence of the deleted direct owner;
- one canonical compiler and the typed standard-dispatch policy;
- receiver-before-arguments-before-call ordering in the shared boundary;
- sole argument-zero projection in the unchanged canonical compiler; and
- TypedArray witness, index coercion and indexed-read order.

The existing `array_at_receiver_policy_structure.rs` now pins the two policy
constructors in the standard dispatcher and the direct branch's selection of
the strict TypedArray entry. The CopyWithin guard changes only the stale end
marker that previously named the deleted wrapper.

The focused CLI fixture `wasm_typed_array_at_argument_evaluation.js` records a
first index expression, ignored second expression and custom iterable spread
before the index object's coercion. The existing Array/TypedArray runtime-kinds
fixture remains the receiver-policy and indexed-read control.

## Verification

On 2026-08-28, the recursive owner target passed `4/4`, the receiver-policy and
neighboring CopyWithin targets each passed `3/3`, and the exact new CLI witness
and existing runtime-kinds control each passed `1/1` against the Wasm backend.
The canonical compiler's source hash remained
`7e4346ef5dac8e59cf58a832157a442c5dc3315e55de56a4b5f601c53aafd33b`, and the
complete Rust source census contains no `emit_array_at_method_call`. Targeted
Rust formatting and the scoped diff check passed. The shared `cargo xc`
checkpoint is green. The pinned `index-argument-tointeger.js`,
`coerced-index-resize.js` and `spread-mult-iter.js` controls pass all six
sloppy/strict Wasm-AOT executions with every failure bucket at zero. No broader
Array or Test262 refresh was performed.

## Nonclaims

This closure does not make the direct branch generic, change receiver
classification, alter the shared `at` algorithm, remove a Test262 materializer,
change a published conformance count or claim the Array subtree green. It does
not canonicalize `push`, `concat`, callback methods or another direct route.

# `Array.prototype.concat` direct-entry ownership

Status: implemented for the Wasm-AOT compiler on 2026-08-28. Focused
verification is recorded below; runtime and Test262 controls remain at the
shared checkpoint.

## Single generic direct entry

Static `concat()` lowering continues to recognize a String receiver or a
statically resolved `String.prototype.concat` target first. That branch still
selects `StandardBuiltinId::StringPrototypeConcat`. Every remaining receiver
now delegates directly through `emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeConcat`, preserving the receiver and complete
source argument list.

The deleted `emit_array_concat_method_call` had one caller. It independently
looked up the same Array builtin metadata, evaluated the receiver, materialized
an internal function object, constructed the complete argument vector and
called that known builtin through the generic function-handle boundary. The
shared direct boundary already owns receiver evaluation, abrupt propagation,
complete left-to-right argument evaluation and spread expansion, and direct
entry into the same standard builtin body. Removing the extra path makes a
second Concat entry owner a compile error.

## Preserved Concat algorithm

`compile_array_prototype_concat_builtin` remains the sole standard Concat
algorithm owner and is unchanged. It still owns receiver conversion, Array
species selection and construction, `IsConcatSpreadable`, array-like length,
per-index `HasProperty` and `Get`, typed-array integer-index observations,
target property creation and final length publication in their observable
order.

This closure changes neither the earlier custom Array named-property path nor
the String classification. `push` remains a separate specialized algorithm,
and `spliceFromArray` remains a live extension.

## Durable evidence

`array_concat_algorithm_owner_structure.rs` recursively pins:

- the String-first and generic Array fallback split;
- complete receiver and argument forwarding through both selected builtin
  paths;
- absence of the deleted wrapper and one canonical Concat compiler;
- the exact standard builtin consumer;
- receiver-before-arguments-before-call ordering at the shared boundary; and
- receiver conversion, species lookup/construction, spreadability, length,
  indexed presence/read and target-write order in the canonical compiler.

The existing Concat core fixture remains the finite generic behavior control.
It covers zero, Array, ordinary-object and multiple arguments, a sparse source,
result identity and public builtin metadata. The pre-existing Filter owner
guard now ends at the shared direct-call boundary rather than the deleted
wrapper.

## Verification

On 2026-08-28, the recursive ownership target passed `5/5` and the neighboring
Filter owner target passed `4/4`. Targeted Rust formatting and the scoped diff
check passed. The canonical Concat compiler remained
`fe301d8165ba41828b9e742f7b19a1e49fabcac9dc35c625bb0a84d4ff29a8e9`, and the
deleted wrapper has zero Rust source occurrences.

The exact existing CLI control
`array::run_wasm_backend_succeeds_for_supported_array_concat_core_fixture`
passes `1/1`. The pinned `call-with-boolean.js`,
`is-concat-spreadable-get-order.js` and
`Array.prototype.concat_small-typed-array.js` controls pass all `6/6`
Wasm-AOT executions with every failure bucket at zero. The shared `cargo xc`,
formatting, diff, module-boundary and task-plan checks are green.

## Nonclaims

This closure changes no canonical Concat semantics, receiver classification,
published conformance status or another Array method. It removes no Test262
materializer and does not claim the Array subtree green.

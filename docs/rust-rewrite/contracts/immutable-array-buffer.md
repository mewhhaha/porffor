# Immutable ArrayBuffers

Status: implemented for the Wasm-AOT backend on 2026-09-24 against the
[proposal specification](https://tc39.es/proposal-immutable-arraybuffer/) and
the Test262 `immutable-arraybuffer` feature at upstream `7ab7faf`. The
Test262 runner no longer gates the feature.

## State

Immutability is the `ArrayBufferFlag::Immutable` bit of the private
ArrayBuffer flags word (see the [flag wire domain](array-buffer-flag-wire-domain.md)).
Only two producers set it: `ArrayBufferSliceKind::ToImmutable` for
`sliceToImmutable` and the `transferToImmutable` arm of
ArrayBufferCopyAndDetach. The bit is never cleared, and an immutable buffer is
always fixed-length and never detachable, so a TypedArray's or DataView's
immutability is fixed for its lifetime.

## One predicate

`FunctionBuilder::emit_array_buffer_is_immutable_i32` is IsImmutableBuffer and
the only reader of the bit; `emit_typed_array_buffer_is_immutable_i32` applies
it to a TypedArray's `[[ViewedArrayBuffer]]`. Every write-side algorithm asks
one of the two:

| Algorithm | Owner | Effect |
| --- | --- | --- |
| ValidateTypedArray(`O`, `order`, ~write~) | `TypedArrayWitnessUse::ValidatedMethodEntry { access: TypedArrayAccessMode::Write, .. }` | TypeError before the detached and out-of-bounds checks (step 4 before step 6) |
| ValidateUint8Array(`ta`, ~write~) | `Uint8ArrayCodecAccess::Write` | TypeError |
| SetViewValue | the eleven DataView setters | TypeError before `ToIndex` |
| DetachArrayBuffer | `emit_detach_array_buffer` | TypeError before the key check |
| ArrayBufferCopyAndDetach | `transfer`, `transferToFixedLength`, `transferToImmutable` | TypeError after `ToIndex(newLength)` |
| `ArrayBuffer.prototype.slice` | species result | TypeError |
| `ArrayBuffer.prototype.resize` | receiver | TypeError (no `[[ArrayBufferMaxByteLength]]`) |
| TypedArray `[[GetOwnProperty]]` | `emit_alloc_typed_array_element_descriptor`, direct own-descriptor facts | `[[Writable]]` and `[[Configurable]]` false |
| TypedArray `[[DefineOwnProperty]]` | `emit_typed_array_define_index_property` | ValidateAndApplyPropertyDescriptor against the non-writable, non-configurable element; no coercion |
| TypedArray `[[Set]]` | same-receiver, distinct-receiver, inherited and receiver-side paths | returns false before coercion for every canonical numeric key |

`ImmutableBufferWriter` names each throwing algorithm and owns its message;
the flag projection stays in the predicate.

`TypedArrayAccessMode` is a required field of every method-entry witness, so a
new `%TypedArray%.prototype` method cannot compile without deciding its access.
The writers are `copyWithin`, `fill`, `reverse`, `set` (whose step 5 precedes
offset coercion), `sort`, the Atomics read-modify-write family and `store`, and
TypedArrayCreateFromConstructor with ~write~ for `from`, `of`, `map`, `filter`
and `slice`. Revalidations after user code are `Read` observations
(MakeTypedArrayWithBufferWitnessRecord), as are `Atomics.load`, `notify`,
`wait` and `waitAsync`.

A `[[Set]]` that returns false is a TypeError under a strict Reference and in
every `Set(O, P, V, true)` (for example the generic `Array.prototype` mutators,
which reach an immutable-backed TypedArray receiver through the ordinary strict
write path rather than the direct TypedArraySetElement store), and a silent
no-op for a sloppy Reference.

## Coverage

`crates/lila-engine/tests/aot_immutable_array_buffer.rs` covers buffer state,
detach and transfer rejection, `[[Set]]` with every receiver shape,
`[[GetOwnProperty]]`/`[[DefineOwnProperty]]`, `Object.freeze`, every TypedArray
writer (including zero-length views, species results and `from`/`of`), the
generic Array mutators, DataView setters and Atomics.

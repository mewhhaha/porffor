# Generic Array toLocaleString observable length

Status: implementation and regression contract updated 2026-09-06. Exact-head
Wasm-AOT verification is recorded in the PR, not inferred from the historical
2026-08-24 private-witness checkpoint.

## Specification and compiler boundary

`Array.prototype.toLocaleString` performs ToObject, then LengthOfArrayLike, before
its ascending element walk. LengthOfArrayLike must observe an own or inherited
`length` property on every receiver, including arguments and TypedArray values.
A private storage extent is not a substitute for that observable property read.

If lookup resolves the standard TypedArray length accessor, its existing policy
returns zero for detached/out-of-bounds views and a whole-element extent for
available views. An override may instead return another length, resize/detach
the buffer, or throw. The generic Array method must honor that observation.

The captured bound does not change during element invocation. Indexed reads
remain live, so resize, detachment, mutation and deletion affect later values,
not the number of visits. Nullish values retain the existing empty-field policy.

## Closed owner and direct-method separation

`compile_to_locale_string_builtin` still has exactly two public producers:
`compile_array_prototype_to_locale_string_builtin` selects `ArrayLike`, while
`compile_typed_array_prototype_to_locale_string_builtin` selects `TypedArray`.
The standard-builtin dispatcher maps each identifier to its matching wrapper.

The generic arm calls `emit_array_like_length_snapshot`, shared with flatMap and
map/filter/every/some. It boxes in the current builtin's Realm, reads `length`
with the boxed receiver, propagates lookup exceptions, and invokes shared
ToLength. It must not load private state, construct a TypedArrayViewLocals,
select any TypedArray witness, read HEAP_LEN_OFFSET, or duplicate ToLength here.
Both entries then use one shared indexed-Get dispatcher, which owns Array,
arguments, ordinary/Proxy and live TypedArray reads without a cached receiver
classification in this emitter.

The direct TypedArray arm retains one brand guard, one immutable private view,
and one `ValidatedMethodEntry` witness. Detached or out-of-bounds direct
receivers throw before element processing; ordinary `length` overrides are not
observed. The shared loop does not repeat method-entry validation or acquire a
new bound.

## Durable checks

`typed_array_to_locale_string_witness_structure.rs` bounds both entry arms and
requires exactly one private-state load/view/witness in the shared owner, solely
in the direct arm. The generic arm must contain exactly one shared observable
length call with abrupt propagation. The loop contains one shared indexed Get
followed by abrupt propagation before the nullish check. A separate guard pins
the shared operation's ToObject/Get/propagation/ToLength order. Captured loop
bounds, live read and validated element invocation order,
exact wrapper dispatch and reverse temporary-local release remain checked.

The unchanged CLI fixture retains tracking, fixed out-of-bounds, odd-byte and
detached-view controls. The 23-program explicit Wasm-AOT length target covers
own/inherited length overrides, arguments, ordering and live-buffer regressions.
See [the implementation follow-up](../aot-array-to-locale-string-length.md) for
commands, evidence boundaries and remaining work.

## Historical evidence is not a new pass claim

The 2026-08-24 checkpoint passed the then-four-test witness target, the four-test
invocation target, and the focused CLI fixture. Three pinned source leaves
(`resizable-buffer.js`, `user-provided-tolocalestring-grow.js` and
`user-provided-tolocalestring-shrink.js`) passed six sloppy/strict executions
through the existing T18 Uint8-only materializer. Those were adapted sources,
not all-constructor or raw-source conformance evidence, and did not establish
observable length shadowing. This follow-up removes the old generic private
snapshot rather than preserving that limitation as an alternate path.

No materializer, source pin, generated status, exclusion or failure expectation
is changed. The [element-Invoke follow-up](../aot-array-to-locale-string.md)
retains these length guarantees while making every non-nullish element use the
validated invocation protocol. Locale formatting, broader object-model and full
current-pin conformance work remain separate.

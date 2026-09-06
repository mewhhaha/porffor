# TypedArray `toLocaleString` buffer witness

Status: normative theory, integrated implementation, independent review and
capped focused verification complete for the Wasm-AOT
`%TypedArray%.prototype.toLocaleString` method-entry seam, 2026-08-23.

## Specification boundary

The living ECMA-262 clause for
[`%TypedArray%.prototype.toLocaleString`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.tolocalestring)
and the corresponding
[ECMAScript 2026 clause](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.tolocalestring)
apply `ValidateTypedArray(this, seq-cst)` before evaluating the shared
`Array.prototype.toLocaleString` element algorithm. The TypedArray method then
uses `TypedArrayLength` in place of an observable `Get` of `"length"`.

That ordering fixes the method-entry contract:

1. require a genuine TypedArray receiver;
2. create one TypedArray-with-buffer-witness record through
   `ValidateTypedArray`;
3. derive the complete call's element-length snapshot with
   `TypedArrayLength`; and
4. only then begin separator construction and the per-index element algorithm.

A detached backing buffer or an out-of-bounds fixed or length-tracking view
therefore throws before an element is read or its `toLocaleString` method is
looked up or called. An in-bounds fixed view contributes its stored element
length. An in-bounds length-tracking view contributes the whole-element length
derived from the same observed backing-store byte length; a trailing partial
element is not visible.

The captured length remains the loop bound for the complete call. Growth during
an element invocation does not add visited indices, and shrinkage or detachment
does not shorten the walk. Values are not cached with that length: each
iteration still performs the live integer-indexed `Get`. An index made
unavailable before its turn consequently produces the current integer-indexed
result and is handled by the unchanged shared element algorithm.

This is deliberately different from borrowing generic
`Array.prototype.toLocaleString` onto a TypedArray. That entry performs
`LengthOfArrayLike`; its TypedArray length observation reports zero for an
out-of-bounds view instead of applying the non-generic method's throwing
`ValidateTypedArray` boundary.

## Closed shared-compiler projection

The two public entries remain explicit and closed:

- `compile_array_prototype_to_locale_string_builtin` delegates with
  `ToLocaleStringReceiverKind::ArrayLike`; and
- `compile_typed_array_prototype_to_locale_string_builtin` delegates with
  `ToLocaleStringReceiverKind::TypedArray`.

Both continue to call the sole `compile_to_locale_string_builtin` compiler.
The `StandardBuiltinId` dispatcher must keep the Array and TypedArray builtin
identifiers mapped to their matching wrappers; this lane adds no second
dispatcher and no raw boolean receiver policy.

Only the `TypedArray` arm changes. It must finish its receiver-brand guard
before reading private view state, load exactly one immutable view through
`emit_load_typed_array_private_state`, construct exactly one
`TypedArrayViewLocals` value and consume exactly one live witness with:

```rust
TypedArrayWitnessUse::ValidatedMethodEntry {
    length_local: len_local,
}
```

That witness is the sole owner of the non-generic entry's `ValidateTypedArray`
and `TypedArrayLength` semantics:

1. it observes the backing data pointer and backing byte length once;
2. it distinguishes detachment and fixed or tracking out-of-bounds state
   without mutating the view's stored fixed extent;
3. it routes both failures through the executing builtin's
   current-function-Realm TypeError path;
4. it floors a tracking view's available bytes to whole elements; and
5. it publishes `len_local` from that same cached observation.

The TypedArray arm consumes that element length directly. It may not call
`emit_validate_typed_array_current_byte_length`, call
`emit_typed_array_current_byte_length`, reconstruct the viewed-buffer,
byte-offset, stored-byte-length or bytes-per-element slots independently,
observe backing-store data or length through a parallel helper, divide a byte
length locally, or overwrite the witness-produced `len_local`. The unused
`typed_buffer_tag_local` has no role in the view record or witness and must not
survive this migration. `ValidatedMethodEntry` already expresses the required
policy; this lane adds no `TypedArrayWitnessUse` variant.

## Distinct generic and element-invocation paths

The `ArrayLike` arm remains separate from this direct method-entry policy.
Its [observable length contract](array-to-locale-string-typed-array-buffer-witness.md)
now requires shared ToObject/Get(length)/ToLength for every receiver, including
arguments and TypedArray values. The generic arm does not consume a private
witness. Standard TypedArray accessors still own their non-throwing
out-of-bounds-as-zero behavior when ordinary property lookup reaches them;
length overrides, coercion and exceptions remain observable.

After either entry has produced `len_local`, the shared loop remains one
algorithm. It compares the ascending index with the captured length and then
performs one live read through
`emit_typed_array_or_object_index_read_from_locals` for a TypedArray receiver.
No second method-entry witness belongs inside that loop.

This migration also leaves the existing element-invocation ownership intact.
For each non-nullish element, the compiler must preserve the original tagged
value before any temporary object conversion, perform the `GetV`-equivalent
lookup, and pass the method and original receiver through the private,
non-`Copy` `ValidatedToLocaleStringInvocationLocals` token. The token's sole
consumer remains Proxy-aware, passes the original element as `this` with an
empty argument list, propagates abrupt completion, converts the returned value
to a string and only then appends it. The buffer witness must precede every such
lookup, validation and call. The separate
[`array-to-locale-string-invocation.md`](array-to-locale-string-invocation.md)
contract remains authoritative for that token and call boundary.

## Durable structural regression

The bounded buffer regression is
`crates/lila-aot-wasm/tests/typed_array_to_locale_string_witness_structure.rs`.
It must isolate `compile_to_locale_string_builtin` through the start of
`emit_object_has_array_index_key_in_range_i32`, then isolate the direct
`ToLocaleStringReceiverKind::TypedArray` arm from the generic `ArrayLike` arm.
Counts from unrelated TypedArray consumers or from the generic arm may not
satisfy the direct-entry assertions.

The regression must pin all of the following:

- the two separately bounded wrappers and their exact, swap-resistant
  `ArrayLike` and `TypedArray` projections;
- the completed direct-entry brand guard before exactly one private-state
  load, one `TypedArrayViewLocals` construction, one live witness and one
  `ValidatedMethodEntry` projection;
- no raw validating or current-byte-length helper, direct private-slot load,
  direct backing-store observation, byte-length division, entry-global
  TypeError construction, `typed_buffer_tag_local`, second witness or direct
  `len_local` assignment in the TypedArray arm;
- the generic arm contains no private witness and captures observable length
  through the shared ToObject/Get/ToLength operation before selecting live reads;
- the complete `len_local` writer inventory leaves the witness snapshot intact
  through the shared-loop bound, and the complete `typed_receiver_local`
  writer/use inventory routes both direct and generic TypedArray entries to the
  live indexed-read helper;
- each Array and TypedArray dispatcher identifier has exactly one owner and is
  mapped to its matching wrapper, so an earlier duplicate arm cannot hide the
  reviewed mapping;
- the shared compiler has one `receiver_kind` binding and only the method-name,
  direct-entry and element-invocation consumers, preventing an unreviewed local
  policy inversion between the wrappers and witness branch;
- the existing validated element-invocation token remains downstream of the
  witness and its sole Proxy-aware consumer still receives the original
  element and an empty argument list; and
- every temporary local reserved by the shared compiler is unique, final result
  publication precedes the first release, and the derived release sequence is
  the exact reverse of the reservation sequence.

The existing
`crates/lila-aot-wasm/tests/to_locale_string_invocation_structure.rs` remains
the companion owner of the closed receiver domain, private non-`Copy`
invocation token, exact validation/call order and self-backed created-Realm
TypedArray method installation. The new witness regression must complement,
not duplicate or weaken, those guarantees. Normalized exact sentinels are
appropriate for wiring and ordering; a broad snapshot of the whole shared
compiler is not.

These are source-structure mutation guards. They do not by themselves prove
runtime buffer behavior or Realm identity.

## Focused evidence

The primary exact CLI fixture is
`crates/lila-cli/tests/fixtures/wasm_array_to_locale_string_core.js`, registered
by `run_wasm_backend_succeeds_for_supported_array_to_locale_string_fixture`.
Its shared Array/TypedArray matrix fixes the essential policy split: a
length-tracking view reflects its length at each new call, borrowing the Array
method onto an out-of-bounds fixed view produces an empty string, and invoking
the non-generic TypedArray method on the same view throws a TypeError.

The existing companion fixture
`crates/lila-cli/tests/fixtures/wasm_array_to_locale_string_invocation.js`,
registered by
`run_wasm_backend_succeeds_for_array_to_locale_string_invocation_fixture`,
continues to cover the preserved validated-invocation token: created-Realm
Array and TypedArray method identity, current-function-Realm element-method
TypeErrors, callable Proxy dispatch, original receiver identity and the empty
argument list.

The exact current-pin Test262 checkpoint is four source files and their eight
ordinary sloppy/strict variants:

- `built-ins/TypedArray/prototype/toLocaleString/return-abrupt-from-this-out-of-bounds.js`;
- `built-ins/TypedArray/prototype/toLocaleString/detached-buffer.js`;
- `built-ins/TypedArray/prototype/toLocaleString/user-provided-tolocalestring-grow.js`; and
- `built-ins/TypedArray/prototype/toLocaleString/user-provided-tolocalestring-shrink.js`.

The first fixes in-bounds shrink/grow behavior followed by out-of-bounds
method-entry rejection. The second fixes detached-buffer rejection before the
element algorithm begins. The grow and shrink leaves mutate the backing buffer
during an element invocation: growth must not extend the captured walk, while
shrinkage must not shorten it and later unavailable indices must contribute the
current integer-indexed result. Together with the exact CLI fixtures and
bounded structural guards, they are the focused checkpoint for this seam. They
must be run with the constrained Test262 worker settings used by the surrounding
T17 lanes.

The direct method-entry checkpoint was green as of 2026-08-23 under the shared
eight-core cap. At that checkpoint `cargo fmt --all -- --check`, `cargo xc` and
`git diff --check` passed; the companion invocation structure suite passed
`4/4`, the invocation CLI fixture passed `1/1`, and each of the four current-pin
Test262 leaves above passed both ordinary sloppy and strict variants, for `8/8`
Wasm-AOT executions with every failure bucket at zero under
`--jobs 1 --threads 1`.

The later generic companion migration changes the shared witness structure
target and strengthens the core fixture with odd-byte and detached generic
cases. On 2026-08-24, the current witness structure target passed `4/4`, the
unchanged invocation structure target passed `4/4`, and the strengthened core
fixture passed `1/1`. Its three adapted Array Test262 leaves passed `6/6`
Wasm-AOT variants with every failure bucket at zero. These results verify the
generic companion additions; the direct method's `8/8` Test262 evidence remains
the separate 2026-08-23 checkpoint above.

## Nonclaims

This direct method-entry lane does not own generic
`Array.prototype.toLocaleString`; its later buffer-observation migration is
recorded by the companion contract above. Neither lane changes the shared
indexed-read helper, integer-indexed exotic semantics, separator selection,
locale formatting, element lookup or conversion, Proxy `Call`, the validated
element-invocation token, or the other remaining raw TypedArray validators. It
does not migrate `copyWithin`, `with`, `set`, `slice`, `map`, `filter`,
constructor validation or species-target validation.

The shared witness structurally routes direct-entry failures through the
executing builtin's Realm, and the existing invocation fixture proves that the
created-Realm TypedArray method is self-backed for element-invocation errors.
That fixture does not induce a detached or out-of-bounds failure at the new
method-entry witness. Created-Realm buffer-error prototype identity therefore
remains an explicit runtime nonclaim unless a later phase adds and verifies
that case.

This lane removes no Test262 materializer or harness adaptation, changes no
published conformance count, proves no complete `toLocaleString`, TypedArray or
binary-data subtree, establishes no current full baseline, adds no
SharedArrayBuffer synchronization and does not make the witness a universal
integer-indexed exotic protocol. T17 remains in progress.

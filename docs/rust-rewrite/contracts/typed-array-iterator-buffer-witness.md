# TypedArray iterator buffer witness

Status: normative for the Wasm-AOT TypedArray iterator creation and step seam.

## Specification boundary

The 2026 ECMA-262 algorithms for `%TypedArray%.prototype.{values,keys,entries}`
perform `ValidateTypedArray(O, seq-cst)` before `CreateArrayIterator`. The
abstract operation `CreateArrayIterator` allocates the iterator and its slots;
on every live TypedArray step, `%ArrayIteratorPrototype%.next` performs
`ValidateTypedArrayBounds`, rejects an out-of-bounds view, and derives the
current element length from that fresh cached backing-store observation. Those
operations may throw a `TypeError` in the Realm of the built-in function that
performs them.

The older Wasm emitters reconstructed the private view slots independently at
both boundaries and called `emit_validate_typed_array_current_byte_length`.
That helper reloads the buffer and derives length, but its internally generated
errors use the entry-global error prototype. Created-Realm TypedArray iterator
methods and their Realm-owned `%ArrayIteratorPrototype%.next` therefore could
throw an entry-Realm `TypeError`, even though their function objects already
carry the correct Realm snapshot. The two reconstructions also remained
outside the live witness used by the migrated access, search and callback
families.

## Closed projection

`TypedArrayViewLocals` is the sole immutable view-slot projection. Iterator
creation and iterator stepping both load those five slots once and pass the
record to `emit_typed_array_witness` with
`TypedArrayWitnessUse::ValidatedMethodEntry { length_local }`.

The witness:

1. reads the backing data pointer and cached backing byte length once;
2. reads the length-tracking flag once;
3. distinguishes detached, fixed out-of-bounds and tracking out-of-bounds
   states without mutating the stored fixed extent;
4. routes both invalid states through the current function Realm's TypeError;
5. derives a whole-element length from the same cached byte length; and
6. publishes that length only after validation.

Iterator creation consumes the validation and discards the published length;
iterator stepping consumes the length for its done test. The same closed
variant is deliberate: both are specification `ValidateTypedArray`-shaped
entry points, while generic Array borrowing and integer-indexed property
observation retain their distinct non-throwing witness variants.

The old raw validation helper remains for binary-data operations not migrated
by this seam. The structural regression bounds both iterator bodies and forbids
that helper, direct private-slot reconstruction, or entry-global error emission
there. Adding a new iterator validation policy requires extending the closed
`TypedArrayWitnessUse` domain rather than another boolean or parallel length
calculation.

## Durable regression

The existing TypedArray iterator fixture retains values, keys, entries,
BigInt, detach, resizable growth, odd-byte Uint16 flooring, fixed-view shrink
and permanently-done
coverage. Its Realm matrix additionally invokes another Realm's TypedArray
iterator methods and `%ArrayIteratorPrototype%.next`:

- a buffer detached before iterator creation must throw that method Realm's
  `TypeError`;
- a buffer detached after creation must make `next` throw the iterator-method
  Realm's `TypeError`;
- a fixed view made out of bounds before creation and after creation exercises
  the same two Realm-aware paths; and
- cross-borrowing the foreign method and foreign `next` onto entry-Realm views
  and iterators proves the error Realm follows the executing builtin rather
  than the receiver or iterator creator; and
- neither error may inherit from the entry Realm's `TypeError.prototype`.

These cases make both migrations load-bearing. The existing resize matrix
continues to pin current-length observation and whole-element flooring.

## Deferred verification

While the low-memory current-pin baseline owns Cargo and Test262, this seam is
verified only with scoped formatting, JavaScript syntax, source-structure and
diff checks plus independent read-only review. The centralized ladder later
runs the focused AOT structure test, the existing TypedArray iterator CLI
fixture, all four `%TypedArray%.prototype` iterator leaves, and the complete
current-SHA binary-data matrix.

## Nonclaims

This seam does not migrate the remaining raw TypedArray validators, complete
the universal integer-indexed exotic protocol, change iterator result-object
allocation, add shared-memory synchronization, retire a Test262 rewrite, or
claim a new conformance count. The pre-edit focused iterator leaves were
already green; this closes source ownership and created-Realm error identity,
not a measured baseline failure or T17.

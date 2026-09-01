# Empty-string split walks UTF-16 code units

Status: normative for the Wasm AOT ordinary-string `split("")` seam.

## Semantic boundary

ECMAScript strings are sequences of UTF-16 code units. When
`String.prototype.split` reaches its ordinary string-separator algorithm and
the separator is empty, every result element is one code unit. This differs
from `%StringIteratorPrototype%`, which advances by code point and keeps a
well-formed surrogate pair together.

The Wasm AOT string payload stores UTF-8 with WTF-8 encodings for lone
surrogates. Walking that byte representation by decoded scalar therefore
cannot implement the empty-separator rule: an astral scalar such as U+1F4A9 is
one decoded scalar but two ECMAScript code units. The required result is
`["\uD83D", "\uDCA9"]`, and each element must equal `charAt` at the same
index.

This rule applies only after the surrounding split algorithm has preserved its
observable order:

1. apply `RequireObjectCoercible` to the original receiver;
2. for an object separator, perform `GetMethod(separator, @@split)` and, when
   present, dispatch it with the original receiver and `limit` values;
3. only on the ordinary fallback, convert the receiver with `ToString`;
4. convert `limit` with `ToUint32` when it is present;
5. convert the ordinary separator with `ToString`;
6. return an empty Array when the normalized limit is zero; and
7. return the whole converted receiver when the separator is `undefined`.

The ordinary fallback must not decode the receiver into split elements or
allocate those result elements ahead of those steps.

## Closed local domain

The private `empty_string_split_units` module owns three opaque local types:

- `UnitIndexLocal`, the next UTF-16 code-unit index;
- `UnitLengthLocal`, the receiver's UTF-16 code-unit length; and
- `OneUnitLocal`, the constant range width of one code unit.

Their fields are private to the local-domain module. The split emitter can
only call `empty_string_split_units::emit`; it cannot construct any of the three
types. That private `emit` function owns their temporary-local reservations and
LIFO releases. Because Wasm instructions and the established string helpers
still accept raw local handles, the module projects `.0` throughout its loop and
length work.

None of the three local domains implements `Clone` or `Copy`.
`emit_one_unit_payload` borrows the index and one-unit width, so the loop owner
retains the same index for advancement and retains both handles for the final
LIFO release. Changing that boundary back to by-value use cannot compile unless
someone also broadens an ownership capability. The structure gate rejects both
changes.

The compiler-enforced boundary is narrower and load-bearing:
`emit_one_unit_payload` accepts `&UnitIndexLocal` and `&OneUnitLocal`, not a raw
byte-index handle or `UnitLengthLocal`. Passing either wrong domain to the
one-result materialization step therefore does not type-check. These are
emitter-side types only; the encoded Wasm locals remain `i64` values identified
by the backend's `u32` local handles.

That typed materializer calls
`emit_utf16_code_unit_range_payload_from_locals`, the existing authoritative
operation that preserves an entire astral scalar when both units are selected
and emits three-byte WTF-8 when only its high or low surrogate is selected. The
authoritative range and code-unit-length helpers internally decode the
UTF-8/WTF-8 storage representation. The invariant is therefore not a transitive
ban on scalar decoding: the empty-separator iteration never advances one result
per decoded scalar, and its materialization never treats a raw byte slice as one
split element.

## Loop and limit contract

The loop begins at code-unit index zero. Before each write it stops when either
the code-unit index equals the receiver's code-unit length or the result count
equals the already-normalized limit. Each iteration materializes exactly one
code unit, writes it through the ordinary Array element writer, then increments
both the result count and code-unit index by one.

The empty source produces no elements. Lone high and low surrogates remain
single elements. A valid pair produces two independently observable surrogate
elements, and joining the result with the empty string reconstructs the
original pair.

## Durable witness

`wasm_string_split_utf16_units.js` covers an astral literal, an escaped pair, a
mixed BMP/astral string, the empty source, lone and reversed surrogates, limit
truncation, a boxed receiver, `charAt` parity and join round-tripping. The Rust
structural gate pins the empty-separator branch to the typed one-unit
materializer and pins that materializer to the UTF-16 range operation. It also
excludes direct scalar-decoder and raw-byte-slice calls from the coordinator;
the authoritative UTF-16 helpers remain free to decode the storage
representation internally.

## Nonclaims and deferred gates

This seam does not complete general string-separator matching, RegExp split,
Unicode normalization or case data, String iterators, the complete String API,
the pinned String tree, or T18. It removes no Test262 materializer and changes
no published conformance count.

The focused ownership structure target passes `4/4`. Runtime behavior is
source-equivalent and the existing empty-separator fixture was not rerun for
this ownership-only change. Broad Cargo, pinned split leaves and the complete
T18 verification ladder remain deferred.

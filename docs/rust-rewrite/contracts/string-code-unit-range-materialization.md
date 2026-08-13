# String range extraction materializes UTF-16 code units

Status: normative for the Wasm-AOT `String.prototype.slice` and
`String.prototype.substring` range-extraction seam.

## Representation boundary

ECMAScript Strings are sequences of UTF-16 code units. `slice` normalizes its
two relative indexes and selects `max(to - from, 0)` consecutive code units.
`substring` clamps both indexes to the String length, orders them, and selects
the code units between the smaller and larger index. Annex B `substr` likewise
selects a normalized start and a count of consecutive code units.

The Wasm-AOT payload stores canonical UTF-8 for scalar values and WTF-8 for
lone surrogates. A four-byte astral scalar represents two ECMAScript code
units, so there is no byte offset between its high and low surrogate. The
operation "convert both code-unit indexes to byte offsets and copy the bytes"
cannot represent either half. The existing byte-offset conversion rounds such
an interior boundary to the end of the scalar. Treating a normalized unit
index directly as a byte offset is wrong for every multibyte scalar.

The authoritative operation is
`emit_utf16_code_unit_range_payload_from_locals`. It scans the storage
representation in the UTF-16 domain, preserves a selected complete astral
scalar, and synthesizes three-byte WTF-8 when only its high or low surrogate
is selected. The raw byte-slice operation remains valid for callers that have
proved byte boundaries; it is not a String range operation.

## Evaluation and coercion order

A method-call expression evaluates its receiver expression and every argument
expression before entering the builtin function. The builtin then performs:

1. `RequireObjectCoercible` on the receiver;
2. `ToString` on the receiver;
3. `ToIntegerOrInfinity` on the already-evaluated start argument;
4. `ToIntegerOrInfinity` on the already-evaluated end argument when present;
5. method-specific index normalization; and
6. range materialization.

Finite magnitudes outside signed-64 range are saturated before the subsequent
String-length clamp. This is only an implementation intermediate: very large
positive values clamp to the String length and very large negative values
take the method's negative-index path, instead of trapping during Wasm numeric
conversion.

The optimized direct `substring` path therefore delegates through the same
direct-builtin call boundary as `slice`. That boundary evaluates the receiver
and complete argument vector before entering the one standard builtin body.
There is no inline algorithm selected according to whether an enclosing throw
target happens to exist.

Nullish receiver errors are created from the current builtin function's Realm.
Borrowing another Realm's `slice` or `substring` therefore produces that
Realm's `TypeError`, not an entry-global error.

## Closed local domain

The private `string_code_unit_range` module owns non-`Copy`, `#[must_use]`
local types for the UTF-16 String length, normalized unit indexes, a range
length, and the materializable range. Callers provide only the evaluated
tagged receiver and builtin argument vector. The coordinator itself converts
the receiver, derives its UTF-16 length, coerces and normalizes the arguments,
and constructs the range.

Only the materializable range token can reach the final extraction boundary.
Its consuming materializer calls
`emit_utf16_code_unit_range_payload_from_locals` exactly once and releases the
owned locals. The module contains neither the UTF-16-index-to-byte-offset
helper nor the raw byte-slice helper, so a caller cannot accidentally pass a
byte offset or byte length into visible String range extraction.

`slice` and `substring` are a closed method domain with exhaustive policy
selection. Annex B `substr` already uses the authoritative range operation and
remains a separate normalization algorithm; the structural regression pins
that existing materialization as a non-regression rather than duplicating its
legacy normalization in the new coordinator.

## Durable evidence

The focused product fixture covers high-surrogate, low-surrogate and complete
astral extraction; negative `slice` indexes; swapped `substring` indexes;
multibyte BMP prefixes; lone and reversed surrogates; direct, `.call` and
enclosing-`try` dispatch; finite indexes beyond signed-64 range;
receiver/argument/coercion ordering; created-Realm nullish errors; and parity
with the already-correct `substr` path.

A bounded Rust source-structure test keeps the method domain exhaustive and
the local types private, non-`Copy` and `#[must_use]`. It requires one consuming
authoritative materializer, excludes both byte-range alternatives, pins the
two standard builtin arms and the direct `substring` wrapper to delegation,
pins both shared index normalizers to saturating conversion, and pins `substr`
to its existing UTF-16 range call.

## Cost and deferred verification

Computing the visible String length and materializing a range remain linear in
the source and selected range. The authoritative range scan replaces the two
separate boundary scans used by the former byte-slice path; it does not add a
new asymptotic cost.

While the low-memory current-pin baseline owns Cargo and Test262, this seam is
verified only with scoped formatting, JavaScript syntax, source-structure and
diff checks plus independent read-only review. The centralized ladder later
runs the focused structure test, the product fixture, the existing
slice/substr fixtures, the exact `slice`, `substring` and Annex B surrogate
leaves, and the broad T18 String matrix.

## Nonclaims

This seam does not change the static-name method-dispatch predicate in
`functions.rs`, remove the dynamic-`Function` slice materializer, make a byte
offset represent an interior surrogate boundary, or modify RegExp-integrated
String methods. It does not complete String iterators, normalization, case
data, the full String API, the pinned String tree or T18, and it changes no
published conformance count.

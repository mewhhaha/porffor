# String code-unit access is not a byte slice

Status: normative for the Wasm-AOT `String.prototype.charAt` and
`String.prototype.at` seam.

## Semantic boundary

ECMAScript strings are sequences of UTF-16 code units. `charAt` and `at` each
return exactly one such code unit when their normalized index is in range.
They differ only in index normalization and their out-of-range result:

- `charAt` does not interpret a negative index relative to the end and returns
  the empty String when the index is out of range;
- `at` adds the String length to a negative relative index and returns
  `undefined` when the resulting index is out of range.

The Wasm-AOT payload stores canonical UTF-8 for scalar values and WTF-8 for
lone surrogates. There is no byte boundary between the high and low UTF-16
code units encoded by one four-byte astral scalar. Consequently the operation
"convert `index` and `index + 1` to byte offsets, then byte-slice" cannot
implement either method. For an astral scalar at the beginning of a String it
selects the whole scalar at index zero and an empty byte range at index one.

The required operation is the existing authoritative UTF-16 range
materializer. With a one-code-unit width it emits the selected high or low
surrogate as a three-byte WTF-8 sequence while preserving BMP and already-lone
surrogates.

## Evaluation and coercion order

A method-call expression evaluates its receiver and all argument expressions
before the builtin body begins. The builtin then performs, in order:

1. `RequireObjectCoercible` on the receiver;
2. `ToString` on the receiver;
3. `ToIntegerOrInfinity` on the already-evaluated index argument; and
4. method-specific relative-index normalization and selection.

The optimized direct `charAt` path must therefore materialize receiver and
argument expression values into locals before entering the shared builtin
coordinator. It must not perform receiver `ToString` before compiling the
argument expression. Receiver and index coercions remain inside the
coordinator so the optimized call and the ordinary standard-builtin call
cannot reorder or independently reimplement them.

## Closed local domain

The private `string_code_unit_access` module owns opaque, non-`Copy` local
types for the normalized UTF-16 unit index, the UTF-16 unit length and the
constant one-unit width. Their fields are private. The module exposes only two
named entry points, `emit_char_at` and `emit_at`, which accept evaluated tagged
receiver/index values rather than a caller-provided unit or byte index.

The entry points share a private coordinator. That coordinator alone:

- converts the receiver and index;
- creates and fills the opaque unit-index and unit-length locals;
- applies `at`'s negative-relative adjustment;
- chooses the method's fixed miss result; and
- passes the typed unit index and typed one-unit width to the one-unit
  materializer.

The one-unit materializer calls only
`emit_utf16_code_unit_range_payload_from_locals`. It cannot accept a raw
`u32`, a byte-offset local or an independently-computed length. There is no
caller-supplied Boolean or result-policy value: the named entry point fixes
`charAt` to empty-String misses and `at` to `undefined` misses.

The encoded Wasm locals remain `i64` values named by backend `u32` handles.
The Rust types enforce the emitter-side domain at the load-bearing
materialization boundary; they do not claim that Wasm itself has distinct
local types for byte and UTF-16 indexes.

## Durable evidence

The product fixtures cover literal and escaped astral pairs, mixed BMP/astral
Strings, lone and reversed surrogates, positive and negative `at` indexes, the
distinct miss results, primitive indexed-access parity and empty-split parity.
The abrupt-order fixture records argument-expression evaluation, receiver
`toString` and index `valueOf` in their required order, and makes an argument
expression throw before receiver coercion can run.

A Rust structural test bounds the private module and all three product call
sites. It pins the typed one-unit materializer to the authoritative UTF-16
range helper, excludes byte-offset conversion and raw byte slicing from that
module, and requires the optimized and standard paths to delegate to the named
entry points.

## Adjacent work and nonclaims

This seam does not make the general UTF-16-index-to-byte-offset helper a valid
representation of boundaries inside astral scalars. `slice` and `substring`
still contain adjacent range-extraction paths that require a separate,
coherent migration to the authoritative UTF-16 range operation. This patch
does not close those paths by implication.

It also does not complete `charCodeAt`, `codePointAt`, String exotic indexed
properties, String iterators, Unicode normalization or case data, RegExp/Intl
integration, the complete String API, the pinned String tree or T18. It removes
no Test262 materializer and changes no published conformance count.

Static freeze gates are exact-file `rustfmt --check`, fixture syntax checking,
focused source inspection and `git diff --check`. Cargo, fixture execution,
focused pinned `charAt`/`at` leaves and the broad T18 ladder remain deferred
until the active current-pin matrix releases the shared runtime.

# String repeat count normalization is total

Status: normative for the Wasm-AOT `String.prototype.repeat` count and result-limit seam.

## Semantic boundary

`String.prototype.repeat` first converts its receiver to a String and then
applies `ToIntegerOrInfinity` to `count`. Only the normalized count is tested:

1. a normalized count less than zero throws `RangeError`;
2. positive infinity throws `RangeError`;
3. a zero count or empty receiver returns the empty String; and
4. a remaining finite positive count repeats the receiver, subject to the
   implementation's maximum String size.

The placement of `ToIntegerOrInfinity` is observable. A negative fraction such
as `-0.5` normalizes to positive zero and therefore returns the empty String; it
must not be rejected merely because its pre-normalized Number value compares
less than zero.

The count-to-emitter-local conversion must also be total. A finite Number can
be much larger than the unsigned 64-bit domain. A trapping Wasm float-to-int
conversion turns such a value into a backend trap before the repeat algorithm
can distinguish its two required outcomes: an empty receiver still returns the
empty String, while a nonempty receiver exceeds the implementation limit and
throws `RangeError`.

## One normalization authority

The repeat count path converts the argument with `ToNumber`, propagates any
abrupt completion, and then calls the shared
`emit_to_integer_or_infinity_number_payload_from_number_payload` operation.
It does not reproduce NaN, signed-zero, infinity or truncation rules locally.

Only after that shared operation does the repeat-specific boundary reject a
negative normalized Number or positive infinity. Every accepted normalized
Number is nonnegative and finite. `I64TruncSatF64U` then projects it into the
unsigned emitter-local domain without a Wasm trap.

Saturation is not an observable change to the ECMAScript count. Values above
the unsigned 64-bit range become the largest unsigned local value, which is
enough for the only later distinctions:

- an empty receiver or zero count exits before the implementation-limit check;
- count one returns the original String; and
- every larger count is compared unsigned against the maximum count permitted
  by the receiver's byte length.

Consequently every saturated value produces the same result as its original
finite Number: empty for an empty receiver and the implementation-limit
`RangeError` for a nonempty receiver.

## Error Realm

Both repeat-created `RangeError` objects belong to the Realm of the executing
repeat function:

- rejection of a negative normalized count or positive infinity; and
- rejection when a nonempty repeated result would exceed the implementation's
  maximum String size.

A repeat method borrowed from a created Realm therefore produces that Realm's
`RangeError`, not the entry Realm's intrinsic. Both sites use the common
current-function-Realm error operation. The existing receiver and count
coercion errors retain their own operation-defined provenance.

## Durable evidence

`wasm_string_repeat_core.js` covers positive and zero counts, NaN and primitive
coercions, negative fractions, an enormous finite count on empty and nonempty
receivers, and the existing Symbol abrupt paths. It also borrows a created
Realm's repeat method and distinguishes that Realm's two `RangeError` paths
from the entry Realm.

The Rust structural guard requires exactly one call to the shared
`ToIntegerOrInfinity` emitter, forbids the trapping unsigned conversion, and
pins the repeat-specific rejection to a current-function-Realm `RangeError`.
It separately keeps the empty/zero fast path before the maximum-result check
and pins that second error to the same Realm policy. The standard builtin must
normalize the count before invoking the result materializer.

## Nonclaims and deferred gates

This seam does not change the implementation's maximum String size, padding
methods, general numeric conversion, the UTF-8/WTF-8 representation, Unicode
normalization or case data, RegExp integration, Test262 materializers, the
complete pinned String tree or T18 closure. It changes no published
conformance count.

Static freeze gates are exact-file `rustfmt --check`, fixture syntax checking,
the structural source guard, `git diff --check`, module/task-plan checks and
the README status-artifact guard. Cargo compilation, fixture execution, the
pinned repeat leaf and the broad T18 ladder remain deferred until the frozen
patch is independently reviewed and the active current-pin matrix releases the
shared runtime.

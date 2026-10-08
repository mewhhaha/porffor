# Numeric conversion Realm projection capability

Status: the original projection consolidation was focused-verified on
2026-08-28. The 2026-10-03 shared ToBigInt error repair is implemented in the
invariant-first source batch; compilation and runtime verification are pending.
The historical results below do not verify the current source.

## Scope

This contract owns the internal projection from `NumericErrorRealmSource` into
numeric-conversion Realm access. It does not own the source domain,
function-body classification, runtime-helper catalog, error allocation,
numeric conversion semantics or completion routing.

## Rust invariant

The operation emitter has one private, non-derived
`NumericConversionRealmAccess` domain. Its helper argument and error allocation effects remain distinct:
helper ABI parameter 6 emits the trusted current environment or, on the
fallback row, the source Realm's function context in source bodies and zero
elsewhere (never `current_env_local` read as Realm metadata), while
direct TypeError, RangeError and SyntaxError construction selects the current function's
Realm or the main-Realm runtime fallback. Sharing the access decision does not
combine those effects; it prevents two identical source projections from
silently disagreeing about whether an environment may be read as Realm
metadata.

The sole projection maps `GlobalFallback` to `MainRealmFallback` and maps
`StandardBuiltinEnvironment` plus `NumericConversionHelperArgument` to
`TrustedCurrentEnvironment`. The projection and all four consumers are
exhaustive. The domain supports no clone, copy, debug, equality or default
observation; the focused unit verifies its expected rows through exhaustive
matches rather than equality assertions.

The original follow-up deleted `OutlinedNumericRealmArgument`,
`NumericConversionErrorRealm` and their parallel projection functions. It
changed no helper argument, error call, local, instruction or ordering.

## Shared ToBigInt error ownership

Primitive ToBigInt conversion uses the same closed Realm access decision for
Number rejection and rejection of Undefined, Null or Symbol, and for SyntaxError
from an invalid String. A borrowed builtin therefore creates these errors using
its defining Realm's intrinsic prototype. Ordinary source execution retains the
existing source Realm fallback; a lexical environment is never interpreted as
trusted builtin Realm metadata. Public mutation of the Realm's error constructors
does not affect intrinsic error ownership.

ToPrimitive still runs once with the Number hint before primitive dispatch, and
its thrown values retain identity. The Number-admission policy, conversion
messages, Boolean/BigInt/String successes, current completion returns and
NumberToBigInt RangeError path are unchanged. The private SyntaxError consumer
is exhaustive over the existing domain; no new Realm classification is added.

The authored Wasm-AOT regression target checks entry and borrowed foreign
BigInt and both DataView BigInt setters, invalid primitive and String errors,
public constructor mutation, nested entry Realm calls, hook order and abrupt
identity, plus successful conversions. These sources have not been executed.
The algorithms are grounded in [ToBigInt](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-tobigint)
and [BigInt](https://tc39.es/ecma262/2026/multipage/numbers-and-dates.html#sec-bigint-constructor).

## Historical verification and non-claims

The dedicated structure target passes `4/4`, the exact projection unit passes
`1/1`, and the neighboring ToIndex Realm and conversion-Realm targets pass
`3/3` and `4/4`. The borrowed TypedArray-set CLI witness passes `1/1`, and the
shared `cargo xc` checkpoint is green. The pinned Array-source and
TypedArray-source negative-offset Set controls pass all `4/4` sloppy/strict
Wasm-AOT executions with every failure bucket at zero.

This source-equivalent invariant does not claim a completion-ABI redesign,
numeric-conversion conformance gain, broad Test262 result, Wasm golden result
or published conformance-count change.

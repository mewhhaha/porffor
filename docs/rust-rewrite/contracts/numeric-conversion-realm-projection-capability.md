# Numeric conversion Realm projection capability

Status: implemented and focused-verified, with the duplicate policy authority
removed on 2026-08-28.

## Scope

This contract owns the internal projection from `NumericErrorRealmSource` into
numeric-conversion Realm access. It does not own the source domain,
function-body classification, runtime-helper catalog, error allocation,
numeric conversion semantics or completion routing.

## Rust invariant

The operation emitter has one private, non-derived
`NumericConversionRealmAccess` domain. Its two consumers remain distinct:
helper ABI parameter 6 emits the trusted current environment or zero, while
direct TypeError and RangeError construction selects the current function's
Realm or the main-Realm runtime fallback. Sharing the access decision does not
combine those effects; it prevents two identical source projections from
silently disagreeing about whether an environment may be read as Realm
metadata.

The sole projection maps `GlobalFallback` to `MainRealmFallback` and maps
`StandardBuiltinEnvironment` plus `NumericConversionHelperArgument` to
`TrustedCurrentEnvironment`. The projection and all three consumers are
exhaustive. The domain supports no clone, copy, debug, equality or default
observation; the focused unit verifies its expected rows through exhaustive
matches rather than equality assertions.

This follow-up deletes `OutlinedNumericRealmArgument`,
`NumericConversionErrorRealm` and their parallel projection functions. It
changes no helper argument, error call, local, instruction or ordering.

## Verification and non-claims

The dedicated structure target passes `4/4`, the exact projection unit passes
`1/1`, and the neighboring ToIndex Realm and conversion-Realm targets pass
`3/3` and `4/4`. The borrowed TypedArray-set CLI witness passes `1/1`, and the
shared `cargo xc` checkpoint is green. The pinned Array-source and
TypedArray-source negative-offset Set controls pass all `4/4` sloppy/strict
Wasm-AOT executions with every failure bucket at zero.

This source-equivalent invariant does not claim a completion-ABI redesign,
numeric-conversion conformance gain, broad Test262 result, Wasm golden result
or published conformance-count change.

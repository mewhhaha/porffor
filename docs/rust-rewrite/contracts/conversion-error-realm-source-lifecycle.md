# Conversion-error Realm source lifecycle

Status: source-equivalent Wasm-AOT invariant, shared-checkpoint verified
2026-08-28.

## Closed authorities

`ConversionErrorRealm::{MainRealm, CurrentFunctionRealm}` is the private
authority for TypeErrors created inside shared conversion operations. Its one
borrowed exhaustive projection preserves the outlined ToPrimitive helper ABI:
`MainRealm` serializes as 0 and `CurrentFunctionRealm` serializes as 1.
`ConversionErrorRealmSource::{Fixed, RuntimeHelperArgument}` separately owns
whether an emitter serializes a fixed policy or reads helper ABI parameter 2.

Neither domain can be cloned, copied, debug-formatted, compared or defaulted.
They have no Rust representation attribute or discriminants. Adding a Realm
row requires an ABI word and a direct TypeError policy; adding a source row
requires both an ABI-argument policy and a TypeError policy.

The runtime-helper decoder retains its existing order: compare parameter 2 to
the main-Realm word, then the current-function-Realm word, and trap on any
other value. The Wasm carrier remains an `i64`; the enum closes Rust emission
policy, not arbitrary runtime corruption.

## Current-function phase ownership

The type-owned current-function Realm proof is
`CurrentFunctionRealmPrimitiveLocals`. It carries payload and tag locals only;
its sole producer and sole consumer make the two fixed boundary selections for
ToPrimitive and primitive
ToString. A builtin can move the token between those phases, but cannot carry,
replace or inspect a freely shaped `ConversionErrorRealmSource` field.

This makes a mismatched main-Realm token unrepresentable through the public
backend boundary. Main-Realm wrappers still borrow their fixed named source
directly, while the outlined helper body borrows `RuntimeHelperArgument`.
Internal forwarding seams continue to borrow the source, so silently dropping
a new source row fails in Rust rather than becoming an emitted-Wasm
discrepancy.

## Verification and nonclaims

The dedicated Rust-lexical structure guard owns the declaration and manual
capability closure, the 0/1 ABI table, both exhaustive consumers, the invalid
word trap, all seven live main-Realm producers, the typed current-Realm lifecycle,
the two fixed current-function boundary selections, the one runtime-helper
source and the helper compilation route. The existing
Error.prototype.toString phase unit and its two CLI fixtures are the focused
behavioral witnesses because they exercise ordinary ToPrimitive errors,
outlined helper transport, Symbol ToString errors and foreign-Realm identity.

The structure target passes `4/4`, both exact CLI witnesses pass `1/1`, and the
exact phase unit passes `1/1`. The unit now reads the active
`builtins/errors/prototype_to_string.rs` module together with its neighboring
parent only to retain the existing closing marker; the active module split is
unchanged. The scoped `cargo fmt -p lila-aot-wasm -- --check` and owned-file
`git diff --check` both pass.

Independent review is clean after the guard was strengthened to include the
attribute boundary, all seven live named borrowed seams, the exact phase-token
census, the two fixed selections and the complete local-release lifecycle. The
coordinated workspace formatter, `cargo xc`, diff, module-boundary, and
task-plan checks pass.

This invariant preserves every helper argument, error call, local operation,
instruction and ordering. It does not claim a conversion-semantics change, a
completion-ABI redesign, broader Test262 evidence, a Wasm-golden result or a
published conformance-count change.

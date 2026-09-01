# Thrown error diagnostic kind authority

Status: implemented with focused structure verification, 2026-08-27.

## Scope

This contract owns the native-error name published to the Wasm host diagnostic
globals when the backend creates a native Throw completion. It does not own
error messages, prototype selection, catch routing, user-thrown values or host
rendering.

## Semantic law

A runtime-created native error publishes the name of the same
`NativeErrorKind` used to allocate its error object and select its prototype.
Message-bearing errors publish their interned message beside that name. The
message-less TypeError path publishes `TypeError` and clears the message global.
Publication remains after object creation and before the Throw completion is
made observable.

## Rust invariant

The private paired diagnostic publisher accepts `NativeErrorKind`, not an
interchangeable raw string. It derives the published diagnostic name through
`NativeErrorKind::as_str`. Both message-bearing producers forward the kind they
already use for error-object construction, and the message-less producer names
`NativeErrorKind::TypeError` directly. A producer can therefore no longer pair
one native-error object/prototype kind with another host-visible error name
without an explicit type-authority change.

The bounded structure regression pins the typed private boundary, sole name
projection, exact three-producer census, absence of raw name producers, paired
global stores, and publication order in both message-bearing construction
paths.

## Verification and non-claims

The focused source structure target is the verification owner for this source-
equivalent invariant. Targeted Rust formatting and diff checks cover the owned
files. This does not change emitted Wasm, error names, messages, prototypes,
completion routing or Realm selection.

`emit_capture_throw_error_name` remains the distinct path for arbitrary
user-thrown values because their name and message are JavaScript properties,
not a compiler-owned `NativeErrorKind`. This change does not complete the Error
trees, Annex B or T24.

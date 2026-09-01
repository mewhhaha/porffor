# FinalizationRegistry TypeError domain

Status: implemented with focused structure verification, 2026-08-27.

## Scope

This contract owns the six algorithm-created TypeError categories emitted by
the entry-Realm `FinalizationRegistry` constructor and prototype methods. It
does not own target retention, weak reachability, cleanup jobs, created-Realm
intrinsics or the collector implementation.

## Semantic law

The constructor rejects a missing `new` target and a non-callable cleanup
callback. `register` rejects a target that cannot be held weakly, holdings that
are the SameValue as the target, and a provided unregister token that cannot be
held weakly. `unregister` applies the same token category, while both prototype
methods apply the same missing-`[[Cells]]` receiver category. Receiver checks
remain before argument access, and the three registration checks retain their
target, holdings, token order before cell storage is observed.

## Rust invariant

The private, capability-free `FinalizationRegistryTypeError` is the sole input
to the shared TypeError emitter. Its six variants project the exact diagnostic
through one exhaustive match. The emitter no longer accepts an arbitrary
diagnostic string, so adding an unmodeled category, spelling a diagnostic at a
producer, or bypassing the domain fails to type-check or the bounded ownership
guard. The emitter continues to create the exception from the active builtin
function's Realm and immediately return the current completion.

The structure regression pins the exact derive-free declaration, the complete
message table, all eight producers, the typed emitter boundary, unique message
ownership, register validation order and receiver brand-check order.

## Verification and non-claims

The focused structure target is the verification owner for this source-
equivalent invariant. The neighboring WeakRef implementation was dry-reviewed
for the established current-function-Realm error pattern. This change does not
change emitted Wasm, diagnostics, exception Realm, successful registration or
unregistration behavior.

This invariant does not make targets collectible, schedule cleanup jobs, add
created-Realm `FinalizationRegistry` intrinsics or close T21's weak reachability
blocker. Broad compilation and Test262 execution remain part of the coordinated
workspace checkpoint.

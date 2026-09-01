# Required Resolved-Realm Ordinary Prototype

## Private owner

`GetPrototypeFromConstructor` fallback may select an ordinary-object intrinsic
from the Realm proven by `GetFunctionRealm`. The private
`functions/required_resolved_realm_ordinary_prototype.rs` module owns the
closed selector, its exhaustive realm-slot map, and the typed result lifecycle.

`OrdinaryDefaultPrototype` is the only re-export. Its nine variants are
`Object`, `MessageError`, `String`, `Number`, `Boolean`, `Date`, `Iterator`,
`RegExp` and `Promise`. `%Array.prototype%` is deliberately absent because its
Array-exotic representation uses a separate typed path.

`ResolvedRealmOrdinaryPrototypeLocal` is visible to the parent only so retained
inherent methods can pass its inferred value from the loader to the installer.
Its tuple field remains private to the child, and the parent neither imports nor
re-exports the type. Only the child can construct the witness or project its raw
local.

## Lifecycle

The loader accepts a routed `ResolvedFunctionRealmLocal`, traps if the Realm,
intrinsics record or selected required slot is absent, and returns the sole
typed witness. It never selects an entry-Realm global.

The installer consumes the witness, installs its payload together with the
Object representation tag, and releases the temporary local. Four retained
generic function-construction fallbacks pass the inferred witness directly
between those two operations. The new-target orchestration method additionally
owns `GetFunctionRealm`, revoked-Proxy routing, required loading, installation
and resolved-Realm release as one operation. Its two product callers remain in
the Error-family fallback owner; one unit witness is the third source call.

The recursive caller census is five loads, five installs and three complete
new-target orchestration calls.

## Source-equivalent evidence

The exact 43-line domain/witness block retains pre-extraction SHA-256
`b37af658ad2dae3817a94c070da0305488686510057e9b60d586e2d726cbf9a4`.
The exact 99-line method block retains pre-extraction SHA-256
`c1b1786018f52bee3a8d37f140d0c95319cc88ba783fc8387f7d4ebb44b0e401`.
Together the 142 selected lines retain SHA-256
`6889cb20756041dce60e685cd3f61b9c0ee8af20f6fdc0ec17159c2c5384a8f9`.
Only the loader, installer and witness visibility spelling changes to
`pub(super)` at the child boundary; their effective caller surface and every
method body remain unchanged.

The 147-line child has SHA-256
`39692aaa0f33487df324707b4e7bddfa82d03af873f1dda87ee303844d9c7907`,
and the extraction reduces `functions.rs` from 12,269 to 12,127 lines. The
focused recursive witness and module-boundary audit pin sole ownership, the
narrow re-export, all nine exhaustive offsets, the sole tuple construction,
both projections, method visibility, caller census and retained caller bodies.

This extraction changes no fallback selection, Realm routing, trap, emitted
instruction, representation tag, local lifetime or caller body. The shared
`cargo xc` checkpoint and all six exact constructor units pass. Semantic
goldens remain unrun for this source-equivalent ownership move.

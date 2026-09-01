# Set collection weak-value admission

Status: implemented and focused-verified, 2026-08-27.

## Scope

This contract owns the value-admission difference between `Set` and `WeakSet`
for `add`, `delete` and `has`. It does not own collection receiver validation,
entry storage, equality, tracing or weak reachability.

## Semantic law

All three methods validate the receiver before reading their value argument.
`Set` accepts every ECMAScript value. `WeakSet.prototype.add` requires an
object or unregistered Symbol and creates its TypeError from the active builtin
function's Realm before zero normalization or entry lookup. For a value that
cannot be held weakly, `WeakSet.prototype.delete` and `WeakSet.prototype.has`
return Boolean `false` before entry lookup.

## Rust invariant

The private `SetCollectionKind::{Set, WeakSet}` domain is the only policy input
to the three shared emitters. It deliberately has no equality capability. Each
emitter matches it exhaustively: the `Set` arm emits no weak-value check, while
the `WeakSet` arm emits the existing rejection or early-false sequence. A new
collection kind therefore cannot silently inherit either admission policy.

`SetCollectionKind` remains `Copy` because its existing layout, brand and
offset projections consume it by value throughout the collection backend.
`CollectionAlgorithmTypeError`, which contains this domain, also has no
equality capability.

The bounded structure regression pins the two variants, eleven producers,
three exhaustive policy matches, receiver-before-argument order, add's check
before normalization and lookup, and delete/has's early false return before
lookup.

## Verification and non-claims

The focused structure target passes `4/4`; the neighboring Map get-or-insert
structure target passes `3/3`; and the existing engine regression passes `1/1`.
The exact WeakSet invalid-value leaves for `add`, `delete` and `has`, paired
with Set primitive-value controls for the same three methods, each pass both
sloppy and strict Wasm-AOT variants for `12/12` aggregate. Every reported
parser, early-error, lowering, runtime, Wasm-backend, host-harness, unsupported,
not-implemented, crash and bug bucket is zero. The coordinated `cargo xc`,
rustfmt and diff checks are green.

This change preserves emitted instructions for both existing variants. It does
not implement weak or ephemeron storage, make unreachable WeakSet values
collectible, or claim the full pinned Set/WeakSet trees.

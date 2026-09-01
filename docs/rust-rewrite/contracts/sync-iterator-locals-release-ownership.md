# Synchronous iterator local release ownership

`ReservedSyncIteratorLocals` is the sole release authority for the eleven
temporary locals reserved by a synchronous iterator consumer. Its private
`SyncIteratorLocals` field cannot be constructed or moved out by Array or
`Math.sumPrecise`, and the owner is neither cloneable nor copyable. The release
operation consumes that distinct owner before returning all eleven locals in
reverse reservation order.

`SyncIteratorLocals` is the capability-free protocol view used by GetIterator,
IteratorStep and IteratorValue. The reserved owner exposes only immutable
`Deref`, so those operations can borrow the local identifiers but cannot obtain
or duplicate release authority. Array spread and `Math.sumPrecise` each keep one
reserved owner across acquisition and stepping, then move it once into release.

Array destructuring already owns its larger eighteen-local lifecycle. Its
eleven-field protocol projection can be borrowed by the shared operations, but
its `SyncIteratorLocals` value is not accepted by the release operation. The
type difference prevents a copied destructuring projection from releasing
locals owned by the enclosing destructuring compiler.

This changes only Rust ownership. Local reservation order, protocol evaluation,
error routing, IteratorClose behavior and emitted instructions are unchanged.
The four-test recursive guard pins the two type roles, the private owner field,
immutable dereference, borrowed operations, the two reserve-to-release product
lifecycles and the destructuring projection boundary.

At the shared Batch AG checkpoint, `cargo xc` passed. This guard plus the
destructuring-owner, iterator-error-policy and `Math.sumPrecise` structure
targets passed `18/18`; the exact array-accumulation suspension and
`Math.sumPrecise` CLI witnesses passed `2/2`. The two generated
`yield-spread-arr-{single,multiple}.js` leaves passed all `4/4` sloppy/strict
Wasm-AOT executions with every failure bucket at zero. No semantic golden was
run because this source-equivalent ownership invariant claims no new iterator,
Array, Math or conformance behavior.

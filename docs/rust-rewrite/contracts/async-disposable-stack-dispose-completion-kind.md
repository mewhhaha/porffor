# AsyncDisposableStack disposal completion kind

Status: implemented as a source-equivalent Wasm-AOT lifecycle invariant.

The parked `%AsyncDisposableStack%` disposal walk stores a private closed
`AsyncDisposableStackDisposeCompletionKind::{Normal, Throw}` word rather than
an unlabeled `has_error` Boolean. The domain derives no cloning, copying,
equality, debugging or default capability. Its exhaustive projection is the
sole authority for the stable `Normal -> 0` and `Throw -> 1` heap encoding.

Initialization stores `Normal`. Folding either a synchronous disposer throw or
an awaited rejection stores `Throw` only after publishing the corresponding
tagged error. The suppression path strictly loads the word and compares it with
`Throw` before deciding whether an earlier error must become `[[Suppressed]]`.
The terminal path strictly loads the same word and compares it with `Normal`
before choosing fulfilment with `undefined` or rejection with the stored error.

Both readers receive a private non-`Copy`, `#[must_use]` typed local. The strict
load accepts exactly the two serialized words and emits `unreachable` for any
other value before routing. The comparison consumes the typed local. Direct
raw reads and writes of the completion-kind offset are confined to those typed
load and store operations, so a wrong-domain local or arbitrary integer cannot
be introduced at a product owner.

`async_disposable_stack_dispose_completion_kind_structure.rs` pins the private
two-row domain, exact 0/1 projection, one-offset load/store ownership, strict
unknown-word trap, initialization, suppression and terminal settlement routes.
This boundary preserves the existing emitted words, LIFO walk, Await timing,
single-error identity and nested `SuppressedError` order. It adds no resource
management behavior or broader T15 conformance claim.

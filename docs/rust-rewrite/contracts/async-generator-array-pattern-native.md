# Mixed async-generator Array pattern lifecycle

The checked `AsyncGeneratorArrayDestructuringIr` is a third closed view of the
existing Generator/plain-Async Array-pattern pipeline. Its source states,
complete mixed body, exact suspension and iterator-operation tapes, allocation
inventory and nested cell census are validated before native emission. Iterator
operations remain dedicated statements belonging to that actual storage owner.

The mixed wrapper mints its environment capability from the actual opaque
carrier and restores the previous capability even when emission fails. Storage
uses the original invocation's exact allocated BindingCell and the existing
nullable IteratorRecord field. No JavaScript iterator representation, GC layout,
Reference transport, request queue or IteratorClose algorithm is added.

The original raw-source acquisition runs once, outside this pattern's own close
scope. The body close destination is reconstructed before resumed Yield or
rejected Await injection. Ordinary suspension preserves the same iterator and
cached next method; operation results publish into their existing checked cells.
Protocol, done and value failures set DONE before propagation. Target/default/
Put and injected abrupts instead reach IteratorClose, with nested owners closing
inner before outer. Closing copies the whole pending completion, observes the
return method and its effects, retires the native edge, then dispatches outward.
Incoming Throw keeps its original value when close also throws; Return can be
replaced by a close failure. Array destructuring still uses synchronous iterator
acquisition and close inside an async generator.

The lifecycle controls compile ordinary JavaScript through Engine's Wasm-AOT
path in strict and sloppy modes. They exercise queued Return/Throw/rejected
Await through nested close and awaiting/yielding finalizers, original whole
identity, close-error precedence, GC, cached-next mutation, acquisition/step/
done/value failures without Close, and target/default failures with Close.
Source pattern fixtures independently cover binding/Reference/key/default order.

ForOf, ForAwait and resource-body admission remains separate. Their complete
head and per-key continuation work is not implied by the mixed pattern owner.
Compilation, tests and runtime execution remain deferred until the whole source
batch is ready; these controls are authored but unrun.

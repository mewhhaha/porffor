# Native retained Array-pattern iterator

An ordinary-generator Array pattern consumes one checked
`OrdinaryGeneratorArrayDestructuringIr` statement. Its acquisition entry,
complete body range and exit are the actual source plan. Iterator operations
are dedicated opaque statements; they cannot be buried in an arbitrary
expression or published as a JavaScript iterator value. Step and rest own
checked result cells, while elision has no result cell.

The backend retains the actual GC `IteratorRecord` in the appended nullable
`BindingCellSchema::DESTRUCTURING_ITERATOR_RECORD` field. The same record owns
the original iterator receiver, cached next method and mutable DONE. It is
published once after fresh GetIterator and reloaded on every resumed entry.
No native record is reconstructed from three JavaScript values. The sole
BindingCell constructor initializes this extra edge to null. It is not part
of a JavaScript tag domain or a second object representation.

The private native storage constructor requires the exact allocated binding,
slot and ordinary-generator invocation frame. Every physical cell lookup starts
at `InvocationFrameSchema::INVOCATION_ENVIRONMENT`. Nested blocks, CaseBlocks,
With environments and per-iteration scopes cannot redirect a hidden record
through a stale lexical-hop cache. Step and rest publish their value through
the existing original-invocation owned-cell writer. Internal operations have
an Empty normal statement completion; they preserve the preceding StatementList
value rather than publish a protocol observation or rest Array as source V.

Fresh acquisition remains outside this pattern's close scope. A failure in the
source expression or GetIterator cannot close its unacquired iterator. An
already-active outer pattern still owns an inner acquisition failure. Once
acquired, the actual record is published and the source state advances to the
body entry. Fresh and resumed execution then reload the same edge and rebuild
the whole close destination before compiling the complete resumable body.
Therefore the handler exists before a resumed Yield injects Return or Throw.
Ordinary Yield and delegated done:false return without retiring the edge.

The shared native IteratorStep/IteratorValue owner marks DONE before propagating
an abrupt next call, non-object result, done getter or value getter. Elision
uses its existing no-value step path. Those failures pass through the enclosing
close handler with DONE already true. Target acquisition, a selected default,
private-brand checking and PutValue leave DONE false and require IteratorClose.
Rest target acquisition precedes draining; both ordinary and suspended rest
use the same actual rest Array construction and step algorithm.

Normal completion and resumed abrupt completion meet at the same finalizer.
The backend snapshots the complete pending Completion, advances the exact exit
state, invokes the existing IteratorClose precedence algorithm, copies the
whole result and then clears the retained edge. It pops its own handler before
dispatching the result, so nested failures close inner before outer. Generic
Identifier Reference retirement never clears this iterator field. Local GC
roots are released after the completion and persistent edge have reached their
actual owners.

Native entry and exit discovery consume the new statement's exact range.
Lexical instantiation traverses the entire checked body, including original
binding targets behind the shared destructuring Put owner. Later Let/Const
targets remain in TDZ before acquisition and defaults; resumed execution does
not reinitialize them. The existing complete resumable sequence and Switch
StatementList checkpoints retain their normal value behavior.

Two authored native artifact controls validate actual Wasm types and the GC
record field operations: one consumes all strict/sloppy and With Engine source
cohorts through the parser, IR and native emitter; the other checks nested
patterns, delegated defaults, closures and actual record publication/reload/
retirement in the emitted code. The Engine source cohort owns semantic ordering,
close precedence, cached-next, GC lifetime, TDZ and per-iteration behavior.
Source review and isolated formatting do not establish runtime acceptance.
No compiler, tests or runtime executions have run for this batch.

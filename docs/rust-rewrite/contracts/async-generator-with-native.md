# Mixed async-generator With native lifetime

`AsyncGeneratorWithIr` is a separate checked source owner for a complete With
head and body in an ordinary async-generator FunctionBody. It uses the same
physical native pipeline as `OrdinaryGeneratorWithIr` and `AsyncFunctionWithIr`.
The public JavaScript object model, async-generator activation, Promise jobs,
request queue, InvocationFrame and BindingCell layouts remain the existing ones.
Wasm GC and the experimental Wasmtime feature surface remain required.

The source carrier consumes actual disjoint mixed expression/body regions and
the original allocation inventory. Its head must terminally publish the exact
`ToObject` codomain into its retained invocation cell. Its body must use the
original analyzed one-row WithObject environment with the hidden object cell at
slot zero. The head completes in the outer environment before this record
exists. The constructor compares the whole actual suspension tape, including
Await, Yield and the existing adopted Return operand, to the checked source plan.

The native adapter's execution kind is closed. Only an actual
`AsyncGeneratorWithIr` can mint its region environment capability. Compilation
scopes that capability over the original With pipeline, then restores the
previous capability on both success and emission error. Head statements consume
the existing operand-region compiler, which suppresses a surrounding Loop or
Switch StatementList value and restores the previous compiler context afterward.
The body continues the actual source Empty-completion wrappers and persistent
StatementList context when an enclosing checked owner has one.

Fresh body entry allocates the real Object Environment Record once and copies
the completed boxed head into its original hidden BindingCell. Resumed entry
reattaches that same child of the reconstructed outer record. Existing checked
source ancestry and the original saved-child search provide this reconstruction;
no function Boolean, additional resume state or second saved environment is
introduced. The body's outward cleanup target is rebuilt before a resumed
Await/Yield can inject a whole abrupt completion. Inner handlers, finalizers and
iterator closes run before With leaves its record. The pipeline then leaves the
record, saves the actual enclosing lexical environment, and dispatches the whole
Completion to the enclosing owner.

Identifier assignment continues the original located Reference transport. An
Object Environment selection is captured before the RHS, retained in the
original invocation's private field-10 edge, and consumed once at Put. Changes to
unscopables during suspension cannot replace the selected Environment Record.
SetMutableBinding's original property recheck remains inside that record. The
old compound operand is acquired before the RHS and coerced only afterward.
Normal Await/Yield preserves the edge; committed Return/Throw uses the existing
retirement before handler/finalizer transfer.

The complete owner does not widen foreign ForOf/resource/linear continuations or
pattern-owned suspension. Source and IR validators keep those boundaries
explicit. Strict With remains a real frontend early error. Sloppy source is the
positive domain; no strictness downgrade is used to obtain a passing artifact.

The authored native controls emit the actual shared source fixtures, validate
their experimental Wasm features and original ObjectER/BindingCell/frame/queue
topology, and inspect real allocation, parent and saved-frame instructions. The
separate Engine controls exercise head boxing and closure lifetime, selected
Proxy Reference plus unscopables changes while requests queue, and injected
Return/Throw or rejected Await through inner and outer awaiting/yielding
finalizers with whole reason identity and GC. These controls are authored but
unrun in this source-only batch. Compilation and runtime verification remain a
mandatory joined checkpoint.

Async-generator admission recognizes a labelled complete `AsyncGeneratorWithIr`
as a noniteration control scope. Only a matching named Break is owned by that
scope. An unlabelled Break or a Continue to an outer iteration remains with its
enclosing scope; a matching With label cannot be a Continue destination. The
existing generic label emitter supplies the named exit, and the checked With
cleanup leaves and saves the original enclosing environment before dispatching
that whole completion. No new continuation plan or dispatcher is introduced.
The surrounding resumable StatementList forwards the exact entry and exit
states of the labelled `AsyncGeneratorWithIr`, including nested immediate
labels. It cannot treat that checked owner as a synchronous statement and gate
its resumed body behind the preceding segment. Other unannotated noniteration
labels retain their existing checked-plan requirements.
The original labelled-With fixture remains unchanged at its failing source
site. Additional source controls exercise an outer Continue and a matching
Break through an awaiting finalizer, requiring each finalizer once while the
With record is still active and the outer record restored afterward.
Compilation and native verification remain pending batch application after the
immutable baseline sweep.

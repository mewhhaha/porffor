# Mixed async-generator Switch native pipeline

`AsyncGeneratorSwitchIr` and `OrdinaryGeneratorSwitchIr` enter one closed
Generator/AsyncGenerator native Switch pipeline. Plain async functions retain
their separate selector-continuation algorithm. The checked mixed carrier owns
the complete discriminant, ordered selectors, fallback and source-order bodies,
their exact suspension tape, the original discriminant and CaseBlock value
BindingCells, and one shared CaseBlock lexical environment.

The discriminant completes outside CaseBlock. Fresh CaseBlock entry initializes
all its lexical TDZ cells and functions once, before any selector runs. Resumed
selectors and bodies reattach the same record using the existing checked source
ancestry. Matching uses strict equality on whole retained values. A selected body
commits its actual source state and bypasses later selectors; an unmatched prefix
continues selectors after a default clause before committing the fallback.
Fallthrough executes bodies in source order without re-evaluating selectors or
reinitializing lexical cells.

Only the actual opaque mixed carrier can mint the scoped native environment
capability. The adapter restores its prior capability on success and emission
error. The async-generator queue, Promise adoption, resume-point certificate,
InvocationFrame, BindingCell, GC layouts and Reference transport remain the
existing owners. Wasm GC and the experimental Wasmtime feature set are required.

Discriminant and selector prefixes use the consumed operand-region compiler.
It suppresses the surrounding Loop or Switch StatementList value while retaining
the inactive contexts for whole abrupt retirement. Its finish callback evaluates
the terminal selector inside the same binding scope that owns generated
Await/Yield temporaries. The original/mixed classic-loop test and update and
mixed If condition use the same callback. The complete prior binding scopes,
StatementList context and environment depth are restored even on emission error;
there is no late binding lookup fallback. The body's own checked CaseBlock value
context becomes active only after selection, preserving source Empty declaration
completion across suspension and GC.

The original break and label frames are rebuilt before resumed injection. Outer
Continue, Return and Throw continue the existing whole-completion/finalizer
transport and lexical unwind. Normal exit or Switch break leaves the same
CaseBlock and saves its actual parent. Nested awaiting/yielding finalizers retain
the whole pending result. Selected Identifier References still acquire their
record and old compound operand before the RHS; suspension preserves the private
edge and committed abrupt transfer retires it through the existing owner.

The native artifact controls compile the actual source fixtures and validate
experimental Wasm types plus the original Environment, BindingCell, frame,
queue, pending-completion and Reference topology. Separate Engine controls use
strict and sloppy ordinary JavaScript to exercise queued selector Await, default
order, fallthrough and captured CaseBlock cells; generated terminal values in
mixed If/loop/Switch phases and selector TDZ; and queued Return/Throw or rejected
RHS Await through yielding finalizers with whole reason identity and GC. All
controls are authored but unrun in this source-only batch. Compilation and
runtime verification remain a required joined checkpoint. Foreign iterator,
resource and other independently owned protocols keep their existing source
admission boundaries.

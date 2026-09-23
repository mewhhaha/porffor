# Canonical async Module-entry lifecycle

Module-entry graphs without source-phase requests use private compiled activations.
The graph contains source function identities, local HasTLA kinds and original
ordered Evaluation/Defer requests. All environments exist before any import is
installed; all units instantiate before source evaluation. Top-level declarations
never share an entry/dependency environment or enter Script global bindings.

The original Module parse and early errors remain authoritative. Trusted generated
AsyncArrow positions carry the private protocol; ordinary source spelling cannot
select it or publish its functions in dynamic source candidate tables.

## Activation and ownership

Synchronous owners retain the private generator instantiation boundary. Async
owners have closed Allocate, Instantiate and Execute modes. Allocate initializes
the invocation environment and TDZ/undefined cells only. Instantiate installs
imports, namespaces and hoisted functions, saves resume state 1 and returns with
its body Promise still pending. Execute starts source once. Source Await uses the
ordinary async continuation and PromiseResolve algorithm. The canonical environment
is retained independently of the suspended lexical chain.

Module records distinguish Linked, Evaluating, EvaluatingAsync and Evaluated;
Empty, Normal and Throw completion; and NotStarted, Executing and Completed body
state. A Throw has an independent presence discriminator, preserving `undefined`.
Runtime state and private entry words are strictly decoded. Corrupt words trap;
they never silently select a valid lifecycle path.

## Evaluation, parents and cycles

Evaluate allocates and retains its intrinsic capability before any source body
can reenter it. A repeated request for the same active root returns that capability.
The iterative DFS marks a record Evaluating, derives effective dependencies from
its original phases, then pushes the active component. Evaluation requests retain
their targets. Defer requests gather first asynchronous dependencies through all
request phases, stopping at HasTLA, an active DFS node or a completed cycle.
Actual targets are deduplicated in first occurrence order; distinct targets that
later share a cycle root remain distinct dependency occurrences.

DFS index/ancestor values choose cycle roots at runtime. Each unresolved async
dependency registers a parent occurrence and increments the corresponding pending
count. Positive async orders are assigned monotonically by the graph. Bodies with
no pending dependencies execute immediately; eligible same-cycle peers never wait
for every other async member to finish. A synchronous component closes immediately;
an async component retains per-member body state and its root capability.

A body fulfillment gathers newly eligible ancestors, decrements every registered
occurrence, and executes the eligible list by async order. Synchronous ancestors
are gathered before their bodies execute; asynchronous ancestors stop that gather
until their body fulfills. Rejection propagates the exact value to registered
parents and owned capabilities once. A late sibling must check an aborted parent's
cached Throw before reading its unset cycle root. Static DFS failure caches the
throw only on records actually entered; unrelated completed components remain final.

## Deferred access and imports

Deferred namespace access first runs pure readiness over all requested phases.
Completed cycles stop traversal; active/evaluating async bodies and unfinished
HasTLA records fail readiness with TypeError without changing evaluation state.
After readiness succeeds, Evaluate completes synchronously and its exact settled
outcome is consumed. Namespace identity and canonical binding cells remain cached.

Dynamic imports retain a fresh outer intrinsic Promise, synchronous operand and
option observations, and a load continuation. Evaluation imports wait on the
cached Evaluate record. Deferred imports gather targets, start all Evaluate calls
before attaching any join reaction, and wait for each successful occurrence. A
missing async dependency list resolves in the load continuation without an extra
await. First rejection is retained exactly, including `undefined`.

ModuleBody and ModuleJoin are closed private reaction kinds with captured Realm.
Their registration accepts owned intrinsic Promise records. It performs no mutable
`then`, constructor or species lookup. Private import waits save the current lexical
chain and use the ordinary async-function resume callback through the same raw
record registration. They do not change ordinary source Await semantics.

## Resources and artifact boundary

Per-invocation traversal vectors and explicit frames are bounded by the validated
record count; parent registration is bounded by the number of distinct record
pairs. No shared traversal bitmap or scratch rewind can overlap reentrant source
execution. The existing heap resource limit applies. The implementation traverses
compiled records and calls emitted Wasm functions; it does not parse/evaluate source.

The seven operations are outlined once in a graph artifact. A plain Script reserves
only unreachable index slots with no caller or function-reference edge. Private IR
operations without their graph fail emission. Graph reachability retains source
activations and the intrinsic generator body; intrinsic Promise bootstrap is required
even by a fully synchronous graph. No private operation is a product export.

The [entry completion owner](module-entry-completion.md) consumes the root capability
independently of unhandled-rejection policy. Pending entry at supported host-work
quiescence remains IncompleteModuleEvaluation. Source-phase and Script-entry graphs
retain their separate explicit capability boundaries.

## Verification boundary

`module_instantiation`, `module_import_jobs`, `module_entry_completion`, closed heap
layout checks and `promise_reaction_initialization_structure` cover trusted metadata
and consumers. `aot_module_async_lifecycle` covers live environments, flattened defer
ordering, readiness retries, duplicate cycle-root occurrences, immutable internal
Promise protocols, ordinary Await hooks, dynamic joins, undefined rejection, late
siblings and async resource lifetime. `module_async_runtime_tests` exports private
operations only in copied test artifacts to check effect-free instantiation, pending
body promises, corrupt states, reentrant Evaluate and graphless size/reachability.

The unchanged six audited async-defer failures and all 147 previously passing module
executions are required integration checks. Staged source and formatting checks do
not establish execution success or update the published full-suite baseline.

The concurrent-import regression starts both imports before awaiting either and
checks distinct import promises and the exact `undefined` rejection. A separate
reproducer that performs those awaits inside a `try`/`catch` within synchronous
`for-of` remains unsupported by the generic async-loop lowering path. Supporting
that form requires structured continuation dispatch and restoration of the
iteration environment below suspended block environments. The module lifecycle
change does not claim to repair that independent lowering gap.

Primary algorithms: [ECMA-262 cyclic modules](https://tc39.es/ecma262/multipage/ecmascript-language-scripts-and-modules.html#sec-cyclic-module-records)
and [deferred import evaluation](https://tc39.es/proposal-defer-import-eval/).

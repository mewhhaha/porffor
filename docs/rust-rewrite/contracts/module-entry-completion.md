# Module entry completion

A Module's entry evaluation has its own completion owner. An unrelated rejected
Promise cannot stand in for that result. Canonical Module-entry graphs own an intrinsic Evaluate Promise even when every
body is synchronous. Retained source-phase drivers keep their separate execution
path. The host completion owner is shared by both.

## Trusted entry boundary

The linker records one private entry operation. Canonical graphs
identify their final `ModuleEvaluate` operation. Retained Module drivers identify
their final private synchronous or asynchronous arrow call. The original Module
parse, early errors, private source spans and independent global Script prelude
remain authoritative. The same source spelling in an ordinary Script produces
no entry operation. There is no source-callable intrinsic.

`ModuleEntryEvaluationIr` has private fields and only the trusted linker boundary
can construct it. Its closed kind records synchronous completion or an intrinsic
Promise result. The AOT emitter checks that a Module-entry graph and the private
root operation have the same owner. Dependency collection, function reachability,
global-property planning and throw inference traverse the actual operand.

A synchronous entry yields `undefined` on success. A Promise entry adopts
the exact intrinsic Promise returned by Evaluate or the retained driver, retaining its object
and result through a module-instance global. The passive heap-root inventory
names this persistent root separately from module records; no executable
collector or linear-memory tracing claim is added. Adoption marks that Promise handled
directly; it does not read `.then`, `constructor` or `Symbol.species`, call user
code or create another Promise. An already-enqueued rejection candidate remains
harmless because rejection reporting re-reads `[[IsHandled]]`.

## Checkpoint order and host result

The main checkpoint drains reachable Promise jobs and supported host work,
including `Atomics.waitAsync` timeouts, before sampling the entry:

| State at the checkpoint | Result |
| --- | --- |
| Earlier prelude or instantiation throw | Preserve that exact throw; adoption may not have happened. |
| Synchronous entry completed | Normal `undefined`. |
| Entry Promise fulfilled | Normal `undefined`, regardless of its fulfillment value. |
| Entry Promise rejected | Throw its exact stored payload and tag, including `undefined`. |
| Entry Promise still pending | `IncompleteModuleEvaluation`, a host outcome. |

The host projects the entry before applying `PromiseRejectionPolicy`. Under
`Ignore`, unrelated rejections neither fail the run nor invoke diagnostic
conversion. Under `FailRun`, a settled successful entry remains subject to the
existing background-rejection policy; an actual entry rejection stays primary.
A pending entry remains incomplete even if the background policy would report a
JavaScript rejection. Error-name capture for an entry rejection reads only data
properties and cannot replace the rejection with a diagnostic conversion error.

The checkpoint is a finite snapshot. Existing `FailRun` diagnostic conversions
can create work after the entry was sampled; this change does not schedule a
second checkpoint for such work. The entry status describes the job/host-work
quiescence immediately before those diagnostics, as defined by the
[main checkpoint contract](main-job-checkpoint.md).

## Artifact and engine boundary

Only Module-entry artifacts export the i64 global `module_evaluation_status`.
The wire domain is closed: `NotStarted` (0), `Pending` (1), and `Settled` (2).
Scripts have typed absence through `Option`, rather than an invented Module
status. At a completed main checkpoint, even an earlier source throw changes the
status to `Settled`. A returned `NotStarted`, a wrong export type or an invalid
word is a malformed-artifact trap.

`Pending` is separate from the ECMAScript completion ABI. No ECMAScript completion
kind, runtime value tag or rejection-value sentinel is added. The existing
Normal/Throw transport is consulted only after the host has checked entry status.
`Engine::run_module`, `observe_module`, compiled-unit execution and cached-byte
execution all decode the same optional export. A pending entry returns
`WasmExecutionFailureKind::IncompleteModuleEvaluation`; it is neither a
JavaScript exception nor a timeout, and it has no exception constructor name.
The one-shot API does not retain the Store for later resumption.

Test262 selects `Ignore` for all source goals. Actual Module rejection still
fails positive tests and can satisfy a matching runtime-negative expectation.
An incomplete entry is a Runtime/Bug outcome and can never satisfy a runtime
negative. Agent failures retain their existing aggregate classification.

## Scope and verification

Canonical TLA/deferred execution uses the [async lifecycle](module-async-lifecycle.md)
with runtime DFS, completion capabilities and private reaction kinds. Script-entry
and source-phase graphs retain their explicit admission boundaries. Host rejection
policy never supplies or substitutes the module's evaluation completion.

Focused verification targets:

- `lila-ir --test module_entry_completion` and `--test module_instantiation`;
- `lila-aot-wasm --test module_entry_completion`, plus the closed status-domain
  library test and existing main-checkpoint structure tests;
- `lila-engine --test aot_module_entry_completion` and
  `--test aot_promise_rejection_policy`;
- `lila-engine --lib execution_failure::tests`;
- `lila-test262 --lib module_entry_completion`, plus the existing actual Module
  rejection test and the unchanged Module replay.

Controls cover synchronous success, direct and transitive TLA, rejection before
and after Await, undefined and object rejection values, both background policies,
prelude failure, intrinsic adoption despite tampered Promise properties, settled
host work, pending after host work, cache/compiled-unit paths and ordinary Script
absence. Runtime verification must use the integrated compiler; source staging
alone is not evidence of repaired conformance counts.

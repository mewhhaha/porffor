# Error stack accessors — 2026-10-11

The pinned stage-three getter/setter is implemented in Wasm AOT and the optional
oracle. Typed internal captured text replaces constructor own properties; native
ErrorData branding, exact home-prototype identity, throwing descriptor writes,
Proxy effects and cross-realm errors follow the
[durable contract](contracts/error-stack-accessors.md).

All seventeen focused native controls pass with zero failures or ignores: three
new regression groups (each runs both script modes and both backends), ten existing
Error stringification/SuppressedError/AggregateError controls and four dispatch
ownership controls. Native execution retains one CPU, one libtest worker,
include-ignored and the original thirty-second fixture budgets. Joined
all-feature/all-target workspace types pass in 157.033 seconds. All twelve
repository guards pass.

The complete official selection contains 35 files and 70 exact execution IDs.
Wasm AOT passes 68 and reports two explicit Unsupported/NotImplemented failures:
both modes of `getter-no-error-data.js` construct WeakMap/WeakSet receivers and
reach the existing T05 weak-reference runtime rejection. The optional oracle
passes all 70. Both runs use sixty-second case budgets and one compiler worker;
AOT retains four case workers, the oracle one. Exact manifests, compiler identities,
source pins, executable digests and failure records are retained in the
[receipt](error-stack-accessors-20261011.json). No case was skipped or filtered.

Earlier evidence remains separate: joined types found four emitter integration
errors and then two oracle accessor types; native execution found the oracle's
prototype slot reservation, then the new wrapper's AOT-only result-text assertion.
The corrected wrapper checks structured Boolean completions for both backends.
Dispatch source assertions were updated for the new private module and the exact
direct SuppressedError allocation. The successful source/command/log identities
and every preceding receipt remain recorded.

Actual Wasmtime weak roots/ephemerons, full workspace and pinned Test262 acceptance,
and task closure remain open. This focused checkpoint does not close T24 or justify
removing CONTINUE.md and the task ledger. Canonical publication counts are unchanged.

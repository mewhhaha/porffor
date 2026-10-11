# Typed GC roots and selected With reads — 2026-10-11

The `17492b7a3` retry completes three source-fenced native targets: 355 passes,
seven failures, zero ignores across 362 controls. The worker target passes all
16 original-budget controls. The campaign target has nine passes and six failed
scopes; retained reports show actual five-second AOT worker timeouts. The AOT
library has 330 passes and one repeated With-read body-size failure: nine reads
add 22,401 bytes (5,825 to 28,226) against the unchanged 9,000-byte ceiling.

The Test262 library transcript separately contains 447 passes, but its paused
orchestrator never collected the watcher status; it is interrupted, unaccepted
evidence. All admitted native children finish before source changes. Another
244 complete targets remain unstarted. The retry's complete 248-target inventory,
source seal, original outcomes and byte-exact campaign reports remain in target.
Six campaign directories contain 45 retained JSON files / 886,986 bytes.

## Actual lowering repair

The failed fixture uses statically planned With candidates, rather than the
runtime-environment `EnvironmentIdentifier::Read` route changed in the preceding
batch. Each selected candidate formerly expanded a second HasProperty and
property-read conditional in P. The closed Sloppy/Strict GetBindingValue spec
operations now call the existing complete shared GetValue body after selection.
The separate HasBinding, second observable HasProperty, receiver, strictness,
throw Completion and lazy outer fallback remain intact. The size control keeps
its 9,000-byte ceiling and inspects the actual HasBinding/GetValue call edges.
A product control exercises a selected value over a TDZ fallback, the missing
fallback error, and a throwing second property check through finally. Strict
removed With bindings retain their specific diagnostic; global missing-name
errors retain their distinct diagnostic.

## Concrete local proof

Known `GcLocal<T>` roots declare `(ref null T)` rather than generic eqref. Typed
stores and same-type local reuse therefore prove the heap type to the validator.
Nullable loads need no cast; logical non-null loads retain `ref.as_non_null`.
Clears write null of the root's own declared heap type before release. The two
known-operand constructors use that proof as well. Actual dynamic value and
Completion payloads remain eqref, and dynamic extraction keeps its checked cast.
New module controls validate heterogeneous live roots, same-type reuse, nullable
clears, both non-null refinements, and retained dynamic checked extraction against
the actual recursive runtime schema. Full generated-module validation and actual
cold timings remain mandatory; source inspection is not a performance result.

## Verification and remaining work

Native/default/release/current pinned acceptance and ownership migration remain
open. No deadline, case, runtime profile, engine optimization setting or semantic
admission is relaxed. The original reports remain actual failures. Final workspace all-feature/all-target types pass in 65 seconds; all 469 tooling
controls and 27 prechecks pass, including eight reproducible Intl generators and
the repository guards. The final module-boundary check passes after the standalone
GC fixture uses the public StringConstruction publication owner. Native outcomes
are pending; the first workspace foundation probe catches the catalog
census that must now state 32 expression + 2 backend + 5 statement + 10 gaps =
49 rows. Both independent evidence tests and durable contracts are updated with
that real two-operation addition.

The [machine record](typed-gc-read-repairs-20261011.json) binds the original
attempt and campaign seals. Stale old-source library ELFs are retired under the
Cargo lock after their attempt is sealed; dependencies and R producers remain.

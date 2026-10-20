# Annex B arguments binding time

Authority: ECMA-262 `5345883164f463e87f8b40aca4956157ecba8783`,
FunctionDeclarationInstantiation's normative-optional web-compat insertion and
alternative function declaration evaluation, and Declarative Environment Record
SetMutableBinding. Scriptw implements Annex B web-legacy block functions.

The algorithm excludes `arguments` from new var binding creation at entry;
it does not exclude an eligible block function's evaluation-time assignment.
A formal parameter with the same name still blocks this Annex B copy.
With simple parameters the copy updates the implicit arguments binding.
With parameter expressions the body VariableEnvironment is separate: before
the block declaration a read resolves to parameter arguments; when evaluated,
sloppy SetMutableBinding creates the body binding if it is absent. Parameter
initializer closures keep the original parameter environment, while body
closures observe the newly created binding through their live lexical chain.
An arrow's own variable environment follows the same late-creation rule.

The historical actual `aee6bc109` compiler records Bugs in
`sloppy-script:staging/sm/lexical-environment/block-scoped-functions-annex-b-arguments.js`
and `sloppy-script:staging/sm/regress/regress-602621.js`. Analysis previously
blocked the implicit binding as though it were a formal parameter.
The repair separates copy eligibility from static variable creation, preserves
real independent declarations, and adds an exhaustive function-variable target.
That target uses existing named-record SetMutableBinding at declaration
evaluation. Units containing an arguments function candidate retain named
environment visibility, so reads/writes and escaping closures resolve the live
binding. No source rewrite or fixture exception is used.

The existing IR control keeps both original inputs and its ID, correcting its
implicit-arguments expectation to the normative copy. New controls retain
formal and lexical collisions, unselected blocks, pre/post identity, parameter
initializer reads/writes and closures, and an arrow's outer arguments identity.
No native acceptance claim is made before the coherent batch compiles and the
unchanged original executions and complete affected inventories are checked.

## Current pinned-suite conflict — 2026-10-20

The full refresh retains a red result for
`sloppy-script:annexB/language/function-code/block-decl-func-skip-arguments.js`.
That test's 2017 explanation appends implicit `arguments` to `parameterNames`.
The pinned specification instead leaves formal `paramNames` unchanged and
creates a separate `paramBindings` list containing implicit `arguments`.
Its Annex B condition checks `paramNames`, permitting the selected outer copy.

The unchanged original test fails in both Scriptw Wasm-AOT and Node 24.19.0 /
V8 13.6.233.17-node.51. Both engines pass the two original staging controls
`staging/sm/lexical-environment/block-scoped-functions-annex-b-arguments.js` and
`staging/sm/regress/regress-602621.js`, which expect the outer copy. All three
were replayed with fresh per-test state and the original 60-second deadline;
the native cohort records two passes and one failure. Changing the compiler
to satisfy the 2017 expectation would contradict the pinned algorithm and
the staging controls.

T24 owns this suite/specification discrepancy. Keep the literal failure in
native snapshots, the generated backlog and chart until the suite conflict
is resolved. No compiler change, vendored-test edit or skip was made, and no
upstream report has been sent.
[Receipt and commands](../test262-arguments-conflict-20261020.json) ·
[Original tests, native snapshots, V8 output and pinned specification](../../../test262/snapshots/annex-arguments-triage-20261020.tar.gz).

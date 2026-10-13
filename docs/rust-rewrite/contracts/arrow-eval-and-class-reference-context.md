# Arrow eval context and class reference strictness

Authority: ECMA-262 `5345883164f463e87f8b40aca4956157ecba8783`,
PerformEval, ResolveThisBinding, GetNewTarget, MakeSuperPropertyReference,
GetThisEnvironment, Function Environment Records and strict mode code.

The actual historical `aee6bc109` replay records Bugs in both modes of
`staging/sm/class/newTargetEval.js`, null-reference Crashes in both modes of
`derivedConstructorArrowEvalNestedSuperCall.js` and unreachable-reference Crashes
in both modes of `superPropEvalInsideArrow.js`. The source arrow contains only
an eval call; the lexical context use occurs in its statically prepared source.
Analysis previously requested lexical captures only for literal this/new.target/
super expressions in the arrow's own syntax. Prepared eval therefore received
undefined new.target, a missing home object or missing derived-constructor cells.

Potential direct-eval calls now request the same lexical captures as those
expressions. This conservative demand uses the existing syntactic eval test;
the runtime checks actual intrinsic identity before selecting direct eval.
Intermediate arrows propagate the captures through existing analysis. Script
arrows do not invent a new.target binding. Prepared eval arrows retain their
separate typed DirectEvalExecutionContext. Existing shared derived cells retain
the initialization/ReferenceError boundary before super and the actual this
after super. Native controls cover called/constructed functions, nested/escaped
arrows, nested eval, method receivers/home objects, shadowed eval and derived
initialization. Existing escaped arrows created by eval remain unchanged.

The same actual compiler records a Bug in
`sloppy-script:staging/sm/class/strictExecution.js`. Class heritage and computed
element names are strict code while evaluating in the enclosing activation.
Reference strictness previously came only from that activation's owner plan.
A scoped lexical strictness override now covers class evaluation and is restored
after it, including nested classes. Reference IR and direct-eval preparation
retain the strictness at creation; called ordinary functions keep their own
strictness. Paired native controls cover class declaration/expression keys and
heritage, direct eval in a key, and sloppy references after nested classes.

Class evaluation must also make every function created by heritage/computed-name
expressions strict. Analysis now uses a scoped strict scan context for its
callable metadata producers, retaining the enclosing activation's own mode.
`sloppy-script:language/statements/class/strict-mode/arguments-callee.js` remains
in the historical cohort; its observed Bug exposes the previously
sloppy arguments object of a function created in class heritage. IR and native
controls cover function metadata/arguments poison plus callables before/after
class evaluation, preserving the strictness restoration boundary.

Actual prepared `8cb35ae74` passes all177 frontend lib controls, retaining all172
prior IDs and adding five parser controls. Its IR test/type graph passes after
restoring three missing tracked sparse fixtures. These are frontend/type
verdicts; none establishes JavaScript/Wasm acceptance for these repairs.

These are prepared repairs. The complete affected inventories and the unchanged
original failing executions must pass through the actual next compiler before
an acceptance claim. Original crashes, nonpasses and all prior passing IDs stay
in the evidence.

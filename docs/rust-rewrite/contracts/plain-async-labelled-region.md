# Plain async owning labelled regions

This source-only T14 proposal is based on the approved joined source manifest
`24671d7bb6e0c923b403642f3a6616da1f4bf587a62cfb9aa1ba38dc3d62735b`.
It is prepared in a separate TARGET tree. The accepted source and MAIN remain
immutable. Compilation, Engine execution and Test262 verification are pending;
this proposal closes no conformance milestone or published status count.

A non-loop label with a suspension must own the complete region, because its
matching break can bypass an ordinary await or awaited-condition while.
Lowering captures the plain async entry before lowering the body and checks
whether that activation's counter advances. Nested-function lowering does not
advance it. The owner uses the existing direct-await, block, conditional and try
continuation plans. `AsyncFunctionLabelledPlanIr` privately
checks that the body has advanced past entry and reserves `bodyExit + 1` as a
fresh region exit. State reversal, a body without a continuation and overflow
cannot form a plan. The label's metadata exposes its own entry and exit to the
enclosing statement sequence. It does not forward a child's exit.

The emitter guards the region by `entry <= activationState < regionExit`, opens
one label break block, and compiles the existing body. It writes `regionExit`
after that block's end. Normal completion and a matching labelled break reach
this epilogue. A break before any child executes still commits the exit. An
Await returns the function before the epilogue and resumes with its retained
state inside the range. Return, throw, a break to an outer label, and any valid
ancestor continue route bypass the inner epilogue and retain their destination's
existing ownership.

The existing labelled-target stack includes the block frame in pending Break
dispatch. An awaited finally captures the original completion, resumes its own
states, and restores or replaces that completion. A restored matching Break
then branches to the label epilogue. If finally returns or throws, its replacement
completion bypasses the epilogue. A label wholly inside finally can consume its
own Break while the outer completion remains retained in the activation.

Block environments retain their existing entry, suspension and exit rules. A
normal path leaves the environment through the block epilogue. An early break
uses the existing branch-target lexical depth to unwind environments before it
reaches the label epilogue. Captured cells remain retained by their closures.
The opaque exhaustive same-activation census ignores nested function bodies.

Direct label chains around an awaited `while`, resumable for-of, or disposable
loop retain their loop owner and continue target. Direct labels around the
established `GeneratorLoop` IR for an eager-head loop with a directly awaited
body now expose that loop's own entry and exit, including label chains; they do
not receive a non-loop region owner. The existing break/continue restrictions
on those source loop bodies remain explicit. The immediate labelled emitter
refuses a resumable non-loop body in a plain async function, so an IR
construction that omits the new annotation produces a compiler error rather than
skipped work. `SynchronousLoopBodyIr` rejects a labelled continuation owner.
Existing statement traversals still visit the same label body; public summary
serialization is unchanged by the added continuation metadata.

For `await 0; outer: { while (await true) { break outer; } } await 0;`, the first
Await advances `0 -> 1`, the while owns entry/ready/exit `1/2/3`, the label owns
entry/body-exit/exit `1/3/4`, and the following Await advances `4 -> 5`. For an
early `break outer` before that while, the label commits its own exit directly;
no condition operand or Promise suspension is evaluated. For a bypassing break
inside try/finally, the awaited finalizer completes before the label commits.
The exact initial-state traces are retained in the proposal's `DESIGN-TRACE.json`.

The admission boundary stays narrow. This owner applies to plain async non-loop
labelled regions whose supported body advances the current continuation counter.
Direct awaits and existing Block/If/Try owners are supported, as are awaited
while conditions with their checked eager bodies. Generator/async-generator
ownership, branch-sensitive condition staging and suspending while bodies retain
their existing boundaries. Switch selection needs a separate owner: the shared
source-suspension visitor refuses explicit or implicit suspension in its
discriminant, selectors or cases before they lower. Nested function bodies stay
opaque. This is a Wasm-AOT capability diagnostic for valid JavaScript, not a syntax
early error. Existing enclosing eager/for-await/for-of loop
restrictions are preserved; this proposal does not claim new ancestor-loop
suspension support.

Future verification must compile this exact independent source, run the complete
Async inventory and meaningful new paired Engine controls, retain all prior
implicit-suspension/switch rejection controls, run the unchanged selected primary
cases, and finish the normal broad checkpoint. Source formatting and source
inspection cannot establish emitted execution correctness.

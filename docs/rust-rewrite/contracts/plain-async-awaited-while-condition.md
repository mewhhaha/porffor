# Plain async awaited `while` conditions

This integrated source extension broadens the existing plain-async awaited-condition
owner to the branch values already supported by straight-line expression
staging: ternary arms, logical RHS values and checked property-only optional
tails. It is based on current MAIN ref40, including the reviewed optional-value
owner. This extension is authored source only: it has not been compiled or
executed, and it changes no published conformance counts. T14 remains in progress.

The parser and early-error stages retain the source statement. An awaited
condition is selected only for a plain async function with a syntactic Await in
its condition and an eager body. `EagerAsyncWhileBody` checks the current
activation's source suspension inventory, including implicit `await using`,
`for await` and async-disposal iterator heads. Nested function bodies remain
opaque because they own separate activations.

The lowerer keeps `loop_depth` intact. A private `WhileCondition` branch context
is installed only while staging this checked condition and records its exact
loop depth. The classifier and ternary/logical/optional factories consume the
same admission decision. The context is restored even when staging refuses,
before body lowering. Every child function and generated function starts with
`ScriptLowerer::new`'s ordinary context; flow-fact transfers do not copy the
condition context. Other loop heads and bodies cannot borrow this authority.
Awaited logical assignment is outside this condition scope. A joined
assignment extension must independently require `loop_depth == 0` and refuse
these references before any condition prefix or continuation state is created.

`AsyncFunctionWhileConditionIr` remains the sole checked owner consumed by the
backend. Its private constructor accepts only eager statements, contiguous
`AsyncAwait` statements and recursively associated `AsyncFunctionIf` value
prefixes. Each resumable value arm must own its lexical container, start at its
selected branch entry and finish at the exact ready state encoded by that
branch plan. The shared `sequence_exit` census must also equal the condition's
ready state. A merely in-range arm cannot claim unused states. General
Try/Switch/class/nested-loop continuation spans and unowned blocks hiding Await
remain refused; the general span census alone is intentionally insufficient.
The body must independently pass `SynchronousLoopBodyIr`, and checked arithmetic
keeps the loop exit distinct from Ready, including overflow refusal.

A statically nullish optional tail can erase every actual Await. The existing
value-result owner then emits an ordinary eager If, and the condition may have
`ready == entry`. This eager prefix still belongs to the restartable condition;
it must not be hoisted before the loop. The owner allocates a fresh exit even
when the prefix is empty or eager, so following async statements retain their
own activation-state segment.

The existing Wasm emitter uses the plain async activation, `AsyncAwait` reaction
and Promise jobs. The If dispatcher tests a selected value branch once and
resumes only that arm. Activation-owned slots retain completed left GetValue,
optional bases/intermediate property reads, invocation callees and receivers.
After the prefix joins, ToBoolean selects the eager body or loop exit. Normal
body completion and continue reset the condition entry before the back edge;
break and a false condition advance to the exit. Skipped branches allocate no
Promise or then-getter work. Each new iteration evaluates its source condition
effects afresh, while a reaction does not repeat earlier effects.

Eager body lowering is unchanged: its consuming carrier temporarily removes
continuation state, lowers ordinary blocks/try/finally/nested eager loops and
restores the condition state. Body block environments remain per iteration.
Return, throw, break and continue retain ordinary completion routes, including
finally replacement. Surrounding async catch/finally owns an operand throw or
awaited rejection and preserves its thrown value and Realm. Existing labelled
regions retain their distinct exits; no new backend state machine or frame
representation is added. Frame, data and summary visitors already descend the
condition prefix and its nested value branches.

Admission remains bounded to this condition owner. Awaited bodies, implicit
body suspension, classic-for and do-while awaited heads, generator protocols,
ancestor-loop composition and broader condition continuation owners retain
explicit compiler gaps. Awaited logical compound assignments, optional Call or
private links and optional chains used as direct invocation References retain
their independent boundaries. Ordinary synchronous logical assignment is not
changed. This extension adds neither runtime source parsing nor an interpreter.

Five new constructor controls cover nested branch association, unused arm
states, missing/unwrapped arms, a valid nested-loop span rejected by the narrow
condition grammar and eager-prefix/exit overflow. Existing malformed Await
state, overflowing state, hidden suspension and suspending-body controls remain.
Three new source IR controls cover the admitted branch grammar, following-Await
ownership, static-nullish erasure and context isolation into child functions and
other loops. Existing source refusals are updated only for the two newly
admitted logical/ternary conditions; their other boundaries remain.

Three new Engine fixture controls declare six semantic Script modes, strict and
sloppy. Their exact sole outputs are `while-branches:ok`, `while-references:ok`
and `while-abrupt:ok`. Internal assertions cover selected/skipped scheduling,
condition effects and eager captured environments across back edges/continue,
retained optional bases and call receivers, eager erased prefixes, marker
rejection/throw identity and Realm, finally replacement of break/continue, and
following awaits. All controls are authored and unexecuted. Existing Engine
awaited-condition regression sources are unchanged.

The coherent implementation batch must compile once, run the focused new and existing async
controls, then complete the repository's verification ladder. README/T14 source
provenance and any measured conformance refresh belong to the joined batch;
published counts require the normal status publisher.

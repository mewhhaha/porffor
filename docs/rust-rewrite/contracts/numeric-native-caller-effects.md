# Numeric native calls retain their synchronous effects

Status: source is authored. Compilation, emitted Wasm, runtime controls and
pinned acceptance remain unverified. Finish all remaining task source before
verification under a confirmed 4096 MiB aggregate kernel cap, zero swap and
serial defaults. T04, T20 and T26 remain open.

## Concrete caller-flow error

A scalar normal result does not make a native call free of JavaScript effects.
Numeric input conversion, formatter arguments/options, Math coercion or
iteration and Atomics index/value/count/timeout preparation can call user code.
The existing catalog lacked that effect for 62 reviewed native entries. Caller
analysis could consequently retain captured value-kind, property-shape or
array-element facts after a hook changed them.

## Consumed catalog contract

The existing `SYNCHRONOUS_USER_CODE` flag is added to six Number entries, five
BigInt entries, the two coercing global predicates, 36 Math entries and 13
Atomics entries. The existing consumed const validation requires the effect
for that exact reviewed domain. No native body, installer, ordinal, function
identity or normal-result fact changes. Locale and randomness keep their
independent host effects.

The noncoercing partition remains explicit: Number's four type predicates,
Number/BigInt valueOf, Math.random and Atomics.pause. Those bodies inspect tags
or internal slots, or ignore the reviewed extra operands; they do not convert
them. Their const assertions retain the absence of synchronous user effects.
Host parseInt and parseFloat already have exhaustive caller-flow ownership and
need no second production path. Wider catalog coverage and numeric algorithms
remain separate obligations.

The flag reaches the existing acquired-callee invocation analysis. Complete
arguments, including ignored operands and spread, evaluate before native
coercion. Direct and candidate native calls invalidate captured facts through
the same consumed authority. Scalar return knowledge remains valid for normal
completion while earlier hook effects and abrupt completions are preserved.

## Authored controls

Six IR controls cover acquired aliases/full operands; direct and candidate
conversion effects; formatter argument/locale options; all reviewed Math
coercion families and iterator hooks; Atomics preparation and BigInt values;
and preservation across the eight noncoercing entries. Existing controls stay
byte-exact outside the new numeric insertion.

Three finite Engine cohorts pair strict/sloppy Test262-host Wasm AOT, require
Normal(Number(262)) and one exact print line, and select one compilation worker
with a finite timeout. They cover conversion order, hooks that change captured
facts, cached callable-Proxy iteration, done-before-value, acquired receiver and
complete arguments, prior effects during abrupt cutoffs, native error Realms,
borrowed callees in both directions and exact poisoned-global restoration.
Locale goldens select ASCII `en` and explicit grouping options.

The Atomics controls use zero normal wait/waitAsync timeouts and zero notify
counts. They create no asynchronous waiter. The abrupt timeout hook throws
before waiting. These controls exercise preparation effects; they do not
replace the existing shared-agent or atomic revalidation obligations.

No controls, JavaScript parser/compiler, runtime, source guard, numeric oracle
or conformance measurement has run for this batch. Generated status counts and
historical artifacts remain unchanged.

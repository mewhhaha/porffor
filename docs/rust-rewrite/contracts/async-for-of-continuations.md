# Structured synchronous for-of bodies in plain async functions

An ordinary synchronous `for-of` in a plain async function may suspend inside
blocks, conditional branches, and try/catch/finally clauses. It still acquires
and steps a synchronous Iterator Record. Async iterator acquisition and
`for await` are separate owners.

`AsyncFunctionForOfIteratorPlanIr` contains an `AsyncFunctionForOfBodyIr` with a
private constructor and immutable accessors. Its constructor recursively
checks the complete statement tree against the actual plain async dispatcher:

- Each direct await advances exactly one continuation state.
- Async conditionals own disjoint branch ranges and a shared exit state.
- Try, catch and finally clauses retain their entry, reserved exit, and join
  states, including eager clauses without awaits.
- Blocks retain their materialized lexical environments. Ordinary branch,
  loop and label dispatch may contain only statements that do not advance a
  continuation state.
- At least one await belongs to this body. Generator, disposal, module-entry,
  nested resumable-loop and abrupt loop-control owners cannot enter this plan.

The loop exit is the checked successor of the body completion state. The
head's existing validation still derives activation, iteration-environment or
entry-local value storage, validates lexical-pattern initialization, and owns
the Iterator Record slots. Head initialization precedes the body in the checked
statement list. Data, planning, binding and throw visitors traverse that whole
list rather than a first-await split.

Code generation guards iterator acquisition and stepping by the loop entry
state. Every body visit reinstalls the enclosing IteratorClose completion
frame and invokes the shared async statement dispatcher. That dispatcher
reinstalls nested catch/finally owners for both initial entry and resumption;
it must finish inner finalizers before dispatching an escaping completion to
the iterator owner. A handled rejection continues the iteration without
closing. Escaping throw and return retain the existing distinct IteratorClose
precedence rules.

A captured head creates one fresh lexical environment per iteration. After an
await the saved environment may belong to a deeper body or catch scope, so the
iterator owner uses the existing resumable parent-chain restoration algorithm
to recover its child of the enclosing environment. Each inner lexical owner
then restores its own child. The loop writes the saved environment on fresh
entry and after leaving the iteration; it does not overwrite a suspended
inner environment before that inner owner can restore it. The canonical await
path owns rejection value and Realm transport.

Direct break/continue, suspending iterable or assignment-head expressions,
nested resumable loops, and explicit awaits in `for await` bodies remain
diagnosed capability gaps. They require their own continuation/control-target
ownership; this change does not route them through an ordinary dispatcher.

Required verification is `cargo check --release --locked --workspace --all-targets`,
`cargo test --release --locked -p lila-ir --lib async_for_of_body`,
`cargo test --release --locked -p lila-ir --test async_for_of_continuations`,
`cargo test --release --locked -p lila-aot-wasm --test plain_async_sync_for_of_iterator_record_structure`,
and `cargo test --release --locked -p lila-engine --test aot_async_for_of_continuations`.
Retain the neighboring async loop/if, module lifecycle, iterator protocol,
closure, and close-precedence oracles. The new native fixture includes the
unchanged module loop that exposed the gap and requires exact output after
both catches; a missing or swallowed rejection cannot count as success.

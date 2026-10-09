# Structured synchronous for-of bodies in plain async functions

The historical 2026-10-04 local-control source batch passed the ref80 combined
`cargo xc --locked --offline` whole-workspace all-target Rust type check.
Emitted Wasm, focused runtime regressions and broad verification remain pending;
full T14 acceptance remains open.

An ordinary synchronous `for-of` in a plain async function may suspend inside
blocks, conditional branches, and try/catch/finally clauses. It still acquires
and steps a synchronous Iterator Record. The later
[for-await body source batch](plain-async-for-await-body-continuations.md) adds a
closed awaited execution view to the same completed plan and checked body; its
actual Next/Close emitter and environment admission remain distinct.

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
- At least one await belongs to this body. Generator, async-disposal, module-entry
  and nested resumable-loop owners cannot enter this plan. Existing synchronous
  resource scopes retain their non-advancing body and disposal ownership.
- The mandatory constructor proves branch ownership over the complete tree
  before any eager `SynchronousLoopBodyIr` shortcut. Only current-loop unlabelled
  Break/Continue can enter the body; nested loops, switches, labels and
  parameter-initialization trees cannot borrow its targets. For-init statement
  trees are included in that proof. Blocks, eager and checked async If arms,
  Try clauses and existing synchronous resource scopes retain the current
  target. Labels remain refused even when they name the current loop.

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

The current break target remains live through selected completion and close.
Its body-end continue target is consumed before common iteration cleanup: only
Continue with this target's auxiliary identity becomes Normal, then the active
iteration environment is left, state resets to entry, and cached next steps.
Continue never reads or calls return. Break waits for awaited finalizers to
select the actual completion, closes once, and chooses the checked successor of
the body exit so following statements run once. A finalizer may replace a local
branch with another branch, Return, or Throw, including rejection of its Await.
A close error replaces selected Break/Return and preserves selected Throw; the
surrounding catch/finally owns close failures. The existing async pending stack,
reaction transport, control auxiliary slot and Iterator Record are consumed by
this path; no activation layout, ABI, opcode, or Await helper is added.

A neutral private CurrentLoop/NestedStatement domain and AST visitor are shared
by actual async and generator for-of admission. Both checked body constructors
consume that domain; the generator's earlier state proof is unchanged. The
generic blanket loop-control visitor for other owners is unchanged. The async
head validator retains its existing eager binding/prepared-assignment/pattern
and physical storage rules; this batch introduces no new head family.

A captured head creates one fresh lexical environment per iteration. After an
await the saved environment may belong to a deeper body or catch scope, so the
iterator owner uses the existing resumable parent-chain restoration algorithm
to recover its child of the enclosing environment. Each inner lexical owner
then restores its own child. The loop writes the saved environment on fresh
entry and after leaving the iteration; it does not overwrite a suspended
inner environment before that inner owner can restore it. The canonical await
path owns rejection value and Realm transport.

Labelled/nonlocal branches, branches below foreign statement owners, suspending
iterable or assignment-head expressions, nested resumable loops, async-generator
ownership remain diagnosed capability gaps for this synchronous route. The
separate completed awaited view now admits the bounded plain-async body-Await
domain described in its linked contract; its current executable acceptance is
unverified. Resource loop heads retain their existing separate admission.
The existing
`async_for_of_body` constructor and `async_for_of_continuations` lowering targets
contain source controls for local completion through awaited finally and
conditional branches, exact following states, fresh captured heads, preserved
eager Try/resource support, and refusal before the eager shortcut can hide
foreign or labelled branches. These authored controls are not runtime evidence.

Required verification is `cargo check --release --locked --workspace --all-targets`,
`cargo test --release --locked -p lila-ir --lib async_for_of_body`,
`cargo test --release --locked -p lila-ir --test async_for_of_continuations`,
`cargo test --release --locked -p lila-aot-wasm --test structure_async -- plain_async_sync_for_of_iterator_record_structure::`,
and `cargo test --release --locked -p lila-engine --test aot_async -- aot_async_for_of_continuations::`.
Retain the neighboring async loop/if, module lifecycle, iterator protocol,
closure, and close-precedence oracles. The new native fixture includes the
unchanged module loop that exposed the gap and requires exact output after
both catches; a missing or swallowed rejection cannot count as success.

# T08 — Environments, references, control flow and abrupt completion

**Status:** In progress — dedicated lowering/emission modules exist; conformance closure remains

**Parallel group:** Core foundations  
**Depends on:** T04, T07  
**Blocks:** T09, T12-T15, T24

## Current repository state

Environment, reference-adjacent lowering and structured control-flow emitters
now support substantial lexical scope, closure, loop, destructuring and
try/finally behavior. Destructuring identifier writes use one typed, validated
Reference across lowering and emission: it closes the former name-plus-boolean
flag convention, carries global `[[Strict]]`, and defers TDZ and immutable
failures until PutValue. Synchronous plain-generator and `yield*` property
assignments now carry one private `SuspendedPropertyReferenceIr` containing the
evaluated ordinary base/receiver, normalized key and `[[Strict]]`; one
exhaustive AOT consumer persists its operands before suspension and spends its
strictness only on normal resume. In supported class and lexical-class
home-object contexts, `delete super.x` and `delete super[key]` now use a
private, consuming `DeleteSuperReferencePlan`: it sequences current
`this`, the raw computed-key value, and the unconditional ReferenceError through
nested abrupt-propagating materializations, with no key coercion or property
deletion. Plain identifier `=` inside `with` now uses analyzed `WithObject`
environment cursors, stable hidden capture slots, an ordered current/captured
resolution chain and a consuming `WithEnvironmentReferencePlan`. Declarative
records cut off outer objects at their exact position; nested strict functions
carry surrounding objects through the existing closure capture machinery. The
selected object re-runs `HasProperty` after the RHS so strict absence is a
ReferenceError and sloppy Set still observes the recheck. Direct value-position
identifier reads now locate their declarative fallback first and consume the
same typed non-empty Object Environment selection. Resolution continues from
inner to outer, the selected record performs GetBindingValue's second
`HasProperty`, and deletion during `@@unscopables` returns `undefined` in
sloppy code or throws `ReferenceError` in a captured strict function. The raw
innermost-object/read accessors are private or deleted, so lowering cannot
bypass declarative cutoff, outer chaining or the recheck. Object-literal
methods still lack the required home-object context
and remain explicit unsupported debt. Async-generator property assignment remains an explicit
activation-ABI gap, as do private and `super` yield-assignment targets. The
parse-once boundary is landed, several environment/control-flow files remain
large shared hotspots, and the language subtrees assigned to this task have not
been proven zero-failure on a current complete Wasm-AOT matrix.

All four identifier numeric-update forms inside `with` now spend that same
non-empty, non-`Clone`, non-`Copy` Reference plan. A selected branch composes
GetBindingValue's independent `HasProperty`/Get, one closed numeric update and
SetMutableBinding's post-Get `HasProperty`/Set around three compiler-private
materializations. The same binding-object identity therefore survives a getter
that deletes the property; strict nested-function References throw before Set,
while sloppy References recreate the property without falling through to an
outer function, global or Object Environment Record. A durable CLI oracle and
bounded source witness cover the lifecycle, all four prefix/postfix results,
and an `@@unscopables` getter that changes a Number fallback to BigInt before
blocking the object binding; the branch-local update therefore remains Dynamic
while post-expression metadata widens to all runtime tags. Proxy `has` traps
separately delete a previously proven global and create a previously unresolved
global before declining the object binding, forcing one run-time `HasProperty`
guard to reject the former without recreation and admit the latter; a
configurable global also loses its static `proven_present` fact. They pin the
exact 16-file `noStrict` Test262 inventory. At pre-batch commit
`156aeb38b28378e04bb852f8d00679f47b401d34`, the representative prefix-increment
and postfix-decrement strict-reference witnesses each reported `0/1` as
`Runtime/NotImplemented`, with the exact diagnostic
``unsupported in lila wasm-aot first slice: unbound identifier `x```. The
integrated IR invariant is `1/1`, the source-bounded contract suite is `4/4`,
the Wasm lifecycle fixture is `1/1`, and the exact current-source cohort is now
`16/16`; these focused results do not claim the full language subtree or pinned
matrix is green.

Strict global compound assignment and prefix update now retain their computed
payload and tag in reserved locals across PutValue's run-time `HasProperty`
check. The checked write path may use emitter scratch/result locals internally;
passing those same locals as the value previously let `x += 1` or `++x`
compute `1`, then publish or return a helper temporary instead. The strict
DisposableStack constructor fixture carries a focused global-write preamble
that pins both the stored value and expression result for the two IR forms.

The earlier focused IR contract and Wasm execution covering TDZ/default order,
strict and sloppy unresolved writes, and immutable assignment are green. The
suspended-property Reference IR contract is also covered by the central
feature-enabled CLI compile, and its exact generator-suspension Wasm fixture is
green. The delete-super and Object Environment Record read/write structural
units and Wasm fixtures are present, while their Cargo and pinned Test262
execution gates remain deferred to the current integration checkpoint. The
Object Environment seam is intentionally limited to plain assignment, direct
identifier GetValue (including `typeof` operands), identifier numeric update
and eager arithmetic/bitwise compound assignment in scripts and ordinary source
functions. Identifier-call `WithBaseObject`, logical compound assignment,
destructuring/delete operations, generated class/helper contexts and resumable
captured WithObject environments remain explicit debt.

Resumable loops now carry a required closed
`ResumableLoopIterationEnvironmentIr::{StorageOnly, FreshPerIteration}` policy.
For the specialized plain-async array `for-of` path, a captured lexical head
selects the fresh policy instead of being rejected. The corresponding backend
contract allocates the iteration record only after the loop test succeeds,
preserves that exact record across `await`, then restores and persists its
parent before the update and next test. A focused CLI fixture invokes all
capturing closures after the loop and requires six distinct values, with the
synchronous `for-of` shape as its control. This lane is dry-written and
statically checked. The integrated current-SHA checkpoint is green: `cargo xc`,
three source-bounded backend structure tests, two focused IR tests, the existing
resumable-loop Wasm module test, and the six-closure consumer fixture all pass.
The two exact pinned `Array.fromAsync` witnesses report `4/4` under Wasm-AOT;
the complete 95-file leaf was not rerun.

Eager identifier compound assignments inside `with` now use the same non-empty
consuming Reference plan. One private closed operation
separates all six arithmetic and all six bitwise operators from the three
short-circuiting logical forms. An opaque old-value/result/write carrier is
sealed to the applied expression before the plan accepts it, so lowering cannot
transpose the three compiler-private bindings or return before same-base
PutValue succeeds. The durable CLI oracle covers all twelve eager operators,
selected-object identity across getter deletion and RHS effects, strict
post-Get deletion, function/global/outer fallbacks, and observable fallback
mutation, deletion and creation. The bounded source witness owns the exact 44
`noStrict` current-source Test262 files: 33 historical function/global/nested
Object Environment Record cases and 11 modern strict nested-function
SetMutableBinding rechecks. `**=` is included by the closed local operation but
has no additional direct vendored witness. The integrated IR domain test is
`1/1`, the source-bounded suite is `5/5`, the retained numeric-reference suite
is `4/4`, the Wasm lifecycle fixture is `1/1`, and the exact current-source
cohort is `44/44`. The broader modern filename filter also exposed 11 adjacent
global Object Environment cases that remain unsupported; they are explicit
follow-up evidence, not part of this `with`-scope passing claim.

## Objective

Implement spec-correct binding resolution and structured control flow so lexical scope, TDZ, assignment, loops and `try/finally` all share one model instead of feature-specific lowering shortcuts.

## Environment records

Provide explicit IR/runtime support for:

- declarative, function, module, global and object environment records;
- lexical/variable/private environment chains;
- mutable, immutable, deletable and indirect bindings;
- initialized vs uninitialized bindings and TDZ checks;
- global declaration instantiation and restricted global properties;
- `with` object environments and `Symbol.unscopables`;
- per-iteration environments for lexical loop bindings;
- catch environments and Annex B catch/`var` interactions.

Closures must capture cells/environment references, not copies, and capture analysis must remain correct across nested functions, classes, generators and async suspension.

## Reference model

Introduce a typed Reference representation covering:

- binding references;
- property references, including primitive bases and `super`;
- private references;
- unresolvable references;
- strictness, receiver and this-value information.

Implement `GetValue`, `PutValue`, `InitializeReferencedBinding`, `Delete`, `typeof` unresolvable behavior and assignment/update evaluation order through shared operations.

## Control-flow model

Lower statements to structured blocks with explicit completion edges:

- labels, `break` and `continue` target resolution;
- `return` and `throw`;
- `switch` fallthrough;
- `while`, `do`, classic `for`, `for-in`, `for-of` and per-iteration binding creation;
- `try/catch/finally`, including completion replacement and value preservation;
- destructuring in declarations, assignment, parameters, catch and loop heads;
- short-circuit expression control flow and optional chaining.

Wasm branch depth must be derived from a structured control stack, never patched with case-specific constants.

## Correctness focus

- TDZ begins at block entry, including loop heads and default parameter initializers.
- RHS/key/iterator expressions are evaluated in specification order.
- `finally` may override return/throw/break/continue exactly as specified.
- Iterator closing on abrupt loop completion is delegated to T15's shared iterator operations.
- Global `var`, lexical declarations and implicit-global writes obey property attributes and strict mode.

## Acceptance criteria

- Environment and Reference types are explicit in IR and do not depend on variable-name string conventions.
- Nested `try/finally` and labelled-loop tests pass without manual Wasm depth values.
- Closure mutation, per-iteration capture and TDZ tests pass.
- Destructuring abrupt-completion/evaluation-order cases pass across declarations, assignments and parameters.
- `with`/unscopables and global declaration instantiation are either implemented or left as explicit, owned failures—not approximated.
- Related `language/statements`, `language/expressions/assignment`, scope and global tests reach zero failures.

## Required tests

```sh
cargo test -p lila-ir environment_ --quiet
cargo test -p lila-aot-wasm control_flow_ --quiet
cargo test -p lila-engine --quiet
cargo test -p lila-cli wasm_ --quiet
```

Run focused real filters for lexical declarations, destructuring, `for-in`, `for-of`, `try`, labels, `with`, global code and closure capture.

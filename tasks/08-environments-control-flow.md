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
bypass declarative cutoff, outer chaining or the recheck. Non-resumable
object-literal methods and accessors now carry an explicit receiver for each
super Reference and a typed HomeObject-bearing function carrier; their focused
IR/structure/CLI gates and exact five-file `10/10` Wasm cohort are green.
Object-method lexical arrows now have a verified closed owner-role boundary:
the first non-arrow owner either supplies the paired lexical `this` and
HomeObject capability, supplies the distinct derived-constructor activation,
or supplies neither. At clean pre-batch commit `039253d27`, exact
`prop-dot-obj-val-from-arrow.js` and `prop-expr-obj-val-from-arrow.js` were
`0/4` with the object-literal-method Runtime/NotImplemented diagnostic. The
workspace/all-target check, focused IR invariant (`1/1`), bounded structure
executable (`4/4`), Wasm CLI fixture (`1/1` in 19.37s), and both exact files
(`4/4`, zero unsupported/crash/bug outcomes) are green. Direct generator and
async object-method body/parameter controls remain green at `4/4` each and
`concise-generator.js` remains `2/2`; those results do not establish complete
suspension-safe transport. Complete async-generator object-method transport
remains explicit debt, and async-generator property assignment remains an
explicit activation-ABI gap, as do private and `super` yield-assignment
targets. The parse-once boundary is landed, several environment/control-flow
files remain
large shared hotspots, and the language subtrees assigned to this task have not
been proven zero-failure on a current complete Wasm-AOT matrix.

The `Array.prototype.keys` resizable-buffer case no longer replaces
`Array.from(iterator)` with a source-spliced `for-of` collector. Its unchanged
vendored body now reaches the general Array iterator and `Array.from` paths;
both sloppy and strict Wasm-AOT executions pass. A materialization invariant
pins the original body so this T08 source-shape shortcut cannot return.

Identifier `typeof` now distinguishes proven-static absence from run-time
Global Object Environment uncertainty. An arbitrary source call, a tracked
property whose `proven_present` fact was lost, or an unbound fallback selected
through `with` lowers to `TypeOf(GlobalPropertyRead)`; only a name still proven
unresolvable uses `TypeOfUnresolvedIdentifier`. The `with` fallback is always a
run-time read because `HasBinding` and `@@unscopables` can create or delete the
global while choosing that branch. Four focused IR scenarios pin builtin
globals after calls, dynamic/accessor/deleted globals, conditional deletion and
creation during `@@unscopables`; the registered Wasm fixture passes `1/1`.
`cargo xc` is green. The exact `BigInt.prototype.toString` leaf that exposed
the defect moved from `24/26` to `26/26`, and the independent `BigInt.asIntN`
control passes `2/2`. The semantic golden review finds exactly thirteen changed
pre-existing fixtures, each containing a `typeof` site covered by the new
run-time rule; their dumps change only emitted-size summaries, and the focused
fixture is the sole added artifact.

CallExpression ordering now has a typed lowering boundary rather than an
informal convention. `LoweredCallArguments` is `#[must_use]`, clears heap-shape
evidence from every earlier argument after an intervening effect, and can be
consumed only after the caller explicitly names zero, one or two pre-argument
callee/receiver snapshots. Direct and sloppy-default `this` observations are
merged only after arguments. Optional chains analyze each property or getter
before lowering the following call arguments while retaining the already
captured callee identity and invalidating the receiver snapshot on later
effects. Focused IR regressions and the compiled
`wasm_call_argument_snapshot_invalidation.js` witness cover ordinary, private,
optional and constructor calls plus getter/argument order. This closes the
stale local-shape defect without claiming broader control-flow conformance.

Non-resumable numeric update and eager arithmetic/bitwise compound assignment
through a `super` property now use a fused Reference contract and verified
consumer fixture. The private plan retains current receiver, raw key, captured
strictness in the fused IR. The AOT raw/coerced carriers then evaluate and
retain the single super base through GetValue and PutValue; a key coercion that
changes the method's HomeObject prototype cannot redirect either operation. The
fixture pins the exact mutation traces
`key,getA,rhs,setA:3:true` and `key,getA,setA:2:true` with an alien receiver,
all four increment/decrement modes for Number and BigInt, strict failed Set,
and uninitialized-`this` before key/RHS evaluation. At the near-HEAD pre-batch
`b0d1d1300` boundary, the four exact
`language/expressions/super/prop-expr-{getsuperbase-before-topropertykey,uninitialized-this}-putvalue-{increment,compound-assign}.js`
files reported `2/8`; two increment files were `0/4`
Runtime/NotImplemented, uninitialized-`this` compound assignment was `0/2`
Runtime/Bug, and the existing GetSuperBase compound guard was `2/2`. The debug
binary preceded the commit by four minutes, so these are near-HEAD rather than
exact-commit measurements. Post-batch workspace check and `cargo xc`, focused
IR `1/1`, structure `5/5`, compiled Wasm fixture `1/1`, exact cohort `8/8`,
and both adjacent eight-execution order/control filters are green with zero
unsupported, crash or bug outcomes. Logical super assignment, private fields,
suspension and the broader super-expression matrix remain explicit nonclaims.

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

Computed ordinary-property eager arithmetic and bitwise compound assignment now
uses one fused Reference. A private, non-copyable producer plan
owns the evaluated base/receiver, raw computed key and captured `[[Strict]]`;
its consuming operation alone can mint the old-value read, apply one closed
`EagerCompoundAssignmentOp`, and produce the backend carrier. The intended AOT
lifecycle keeps the raw key distinct from the canonical key: evaluate base,
evaluate raw key, reject a nullish base, perform exactly one `ToPropertyKey`
and `[[Get]]`, then lower/apply the RHS and perform `[[Set]]` with the same
base, receiver and key. A false Set result is routed through the captured
strictness, and the expression result is published only after PutValue returns
normally.

The durable CLI fixture covers all twelve eager operators, including `**=` as
a local closed-domain boundary, and makes each Reference phase observable with
Proxy and accessor traces. It also covers base/raw-key/ToPropertyKey/RHS abrupt
completion, nullish ordering, mutation of the raw key during RHS evaluation,
strict and sloppy false-Set behavior, and nonpublication on abrupt completion.
The exact raw Test262 inventory is the 44 physical files
`language/expressions/compound-assignment/S11.13.2_A7.1_T1.js` through
`S11.13.2_A7.11_T4.js`, each executed in sloppy and strict Script mode. At
clean pre-batch commit `ae1bd994b`, a fresh full run measured `22/88`: every
T3 control passed (`22` executions), while every T1, T2 and T4 execution was
`Runtime/Bug` (`66` executions). No source rewrite, matrix mask or
known-failure entry owns the cohort. Post-batch verification is green:
workspace/all-target check and `cargo xc`; the focused IR invariant `1/1`; the
bounded structure executable `7/7`; retained Super, `with`, and global
compound-assignment structures `5/5`, `5/5`, and `4/4`; the compiled Wasm
lifecycle fixture `1/1` in `75.42s`; and the exact raw matrix `88/88`, with zero
unsupported, not-implemented, crash, or bug outcomes. The legacy matrix
evidences eleven operators; `**=` adds no
twelfth Test262 claim. Plain, logical and numeric property assignment,
`super`, private, identifier/global/Object Environment, `with`, suspending RHS
and the broader compound-assignment subtree remain explicit nonclaims.

Computed ordinary-property numeric update now uses the same fused Reference
ownership boundary. `OrdinaryPropertyReferencePlan::numeric_update`
consumes the evaluated base/receiver, raw key and captured `[[Strict]]` into
closed `NumericUpdateOp::{Increment, Decrement}` and
`UpdateReturnMode::{Prefix, Postfix}` domains. The backend contract orders base,
raw key, nullish rejection, one `ToPropertyKey`, `[[Get]]`, one `ToNumeric`,
new-value computation, same-Reference `[[Set]]`, strict false-Set routing and
only then old/new result publication.

The durable CLI fixture covers all eight Number/BigInt prefix/postfix modes,
their stored and returned values, Proxy/accessor receiver identity, raw-key
mutation during `ToNumeric`, every abrupt phase, and strict versus sloppy
false-Set behavior. The exact raw Test262 inventory is:

- `language/expressions/postfix-decrement/S11.3.2_A6_T1.js`;
- `language/expressions/postfix-increment/S11.3.1_A6_T1.js`;
- `language/expressions/prefix-decrement/S11.4.5_A6_T1.js`; and
- `language/expressions/prefix-increment/S11.4.4_A6_T1.js`.

Each file has no explicit flags and therefore contributes sloppy and strict
Script executions. At pre-batch head `0f004c0c6`, a fresh exact run measured
`0/8`, all `Runtime/Bug`, because observable key coercion incorrectly preceded
the required nullish-base `TypeError`. No source rewrite, matrix mask or
known-failure entry owns the cohort. Post-batch verification is green:
workspace/all-target check; the focused IR invariant `1/1`; the new and
retained eager-compound structure executables `7/7` each; the compiled Wasm
lifecycle fixture `1/1` in `60.43s`; and the exact raw cohort `8/8`, with zero
unsupported, not-implemented, crash, or bug outcomes. Eager/logical/plain
property assignment, `super`, private,
identifier/global/Object Environment, `with`, optional chains, suspended
References and the broader update-expression subtree remain explicit
nonclaims.

Plain assignment through an ordinary property Reference now uses a focused
staging seam. The private consuming producer plan owns one evaluated
base/receiver, one raw referenced-name expression, the RHS and captured
`[[Strict]]`. Its backend consumer preserves the exact lifecycle:
base, raw key, RHS, nullish `ToObject` validation, exactly one
`ToPropertyKey`, same-reference `[[Set]]`, strict-false routing and only then
RHS-result publication. The durable CLI fixture makes each boundary observable
with Proxy/accessor receiver identity, RHS-before-coercion key mutation,
nullish and abrupt completion, exactly-once evaluation, strict and sloppy false
Set results, and primitive receivers.

The exact raw Test262 inventory is:

- `language/expressions/assignment/target-member-computed-reference-null.js`;
- `language/expressions/assignment/target-member-identifier-reference-null.js`;
  and
- `language/expressions/assignment/target-member-identifier-reference-undefined.js`.

Each file has no explicit flags and therefore contributes sloppy and strict
Script executions. At clean pre-batch head `eb32c63a`, the two null-base files
were freshly `0/2` `Runtime/NotImplemented`; the undefined-base identifier file
was `1/2`, with strict passing and sloppy reporting `Runtime/Bug`. The selected
baseline is therefore `1/6`. The adjacent
`target-member-computed-reference-undefined.js` and
`target-member-computed-reference.js` controls were each `2/2`. No runner
rewrite, matrix mask or known-failure entry owns the cohort. Post-batch
verification is green: the workspace/all-target check in 15.18 seconds and
cached `cargo xc` in 0.17 seconds; the focused IR invariant `1/1` in 6.85
seconds after an 8.25-second build; the new structure executable `7/7` in 0.01
seconds after a 20.76-second build; retained eager-compound and numeric
structures `7/7` each in 0.22 and 0.02 seconds; and the exact Wasm CLI fixture
`1/1` in 66.90 seconds. The selected three files now pass all `6/6` executions
with zero unsupported, not-implemented, crash or bug outcomes, while both adjacent controls
remain `4/4`. Focused runtime verification removed only the unsupported
`(1).p` property-read assertion from the fixture; both primitive-assignment
oracles remain. These focused results do not claim the broader assignment leaf.
Destructuring assignment, `super`, private,
identifier/global/Object Environment, `with`, optional-chain and resumable
property References remain explicit nonclaims. The closed boundary is recorded in
`docs/rust-rewrite/contracts/ordinary-property-plain-assignment-reference.md`.

Ordinary-property `&&=`, `||=` and `??=` now have their own consuming
Reference carrier rather than decomposing into an independent read and write.
One lowered base/receiver and raw key flow through nullish validation, one
`ToPropertyKey` and `[[Get]]`; the logical branch alone owns RHS evaluation and
same-reference `[[Set]]`, and publishes the RHS only after PutValue completes
normally. As a backend optimization, the shared state retains one boxed target
`O` separately from the original receiver, so primitive getters and setters
observe the primitive while Get and taken Set use the same object target and
canonical key. Eager compound assignment and numeric update inherit that
backend invariant through the shared GetValue transition. Possible writes also
invalidate dependent global-property facts and Array prototype fast paths.

At clean pre-batch commit `04e38f2ba`, the three exact strict
`lgcl-{and,or,nullish}-assignment-operator-no-set-put.js` files measured `0/3`,
all `Runtime/Bug` with `assert.throws expected an error object`. The three
independent `lhs-before-rhs.js` files were already `6/6` across sloppy and
strict execution. The complete selected post-batch inventory is green: all
eight strict false-Set files pass `8/8`, the ordering controls remain `6/6`,
and the three short-circuit controls pass `3/3`; every failure-kind and
NotImplemented/Crash/Bug bucket is zero, with no exact runner rewrite or
known-failure mask. Central verification passed workspace/all-target checking,
the focused IR invariants `2/2`, the new structure executable `6/6`, the three
affected retained structure executables `21/21`, and the Wasm lifecycle fixture
`1/1` in `76.52s`. This is a fourteen-file, seventeen-execution Reference
batch, not a claim that the complete logical-assignment directory or pinned
matrix is green. The normative boundary is
[`ordinary-property-logical-assignment-reference.md`](../docs/rust-rewrite/contracts/ordinary-property-logical-assignment-reference.md).

After that checkpoint, source-only hardening widened implicit-call effect
tracking across base, key, getter, RHS, reflective and Proxy paths; joined
omitted hook formals with `undefined`; and replaced per-carrier copies of the
complete source-function set with a shared immutable hook-target universe.
The follow-up checkpoint ran under an eight-core, low-priority cgroup:
workspace/all-target checking passed; the filtered ordinary-property IR suite
is `49/49`; logical, plain, eager-compound and numeric-update structure suites
are `27/27`; and the Wasm logical-assignment lifecycle fixture is `1/1`. The
complete current logical-assignment leaf now passes `132/132`, with every
failure phase and NotImplemented/Crash/Bug bucket at zero. One new test
originally confused Number-or-Undefined numeric coercion with general
string-capable addition, and one retained structure witness still expected
alias invalidation outside the shared possible-write transaction; both tests
were corrected to assert the actual contracts before their focused reruns
passed.

Ordinary-property writes now derive a closed possible-mutation authority set
for the Global Object and the Array, Number and Boolean prototypes. An unknown
object base conservatively carries every authority; an exact base shape carries
only the authorities whose current canonical shape matches it. Possible writes
invalidate global and prototype facts through exact and joined aliases before a
must-use pending publication may install the post-`Set` own-property fact.
Structural alias reachability follows own data properties, boxed-primitive
payloads and array elements, but not `[[Prototype]]`, which is inheritance
rather than object identity; authority-bearing root aliases are not republished
as fresh shapes. Separately, the analysis prepass now exports summaries without
replacing root-entry global facts or unknown-effect state, nested body lowerers
retain prepass identity, and body inspection no longer mutates the parent's
live facts as though the function had executed. Final lowering and actual
source-call paths replay live invalidation in source order. The complete
`lila-ir` unit suite passes `892/892`; the exact and joined `globalThis`
aliases, refined Number-prototype alias, boxed-Boolean alias, nested
object/array alias and stale-sibling-publication controls each pass `1/1`.
This is conservative compiler-fact soundness, not an object-identity analysis
or a full conformance claim.

The follow-up runtime audit found one separate cache-coherence hole: the main
script owner still lowered `var` arithmetic compound assignment through its
frame binding, even though nested owners read and write the authoritative
global property. A nested call could therefore update `trace`, after which the
main owner's `trace += suffix` resumed from the stale frame value and erased
the nested write. Script-global compound assignment now selects the global-
property read/modify/write IR at every owner. The public IR regression pins
both the nested and main nodes at `1/1`; the existing ToLength abrupt-route CLI
fixture, whose callbacks append to one script-global trace, passes `1/1`; and
the shared `cargo xc` checkpoint is green.

The shared semantic Wasm-golden checkpoint retains the same 646 fixture rows
(648 files including the manifest and largest-function report), with no added
or removed fixture. The accumulated semantic batch changes 452 fixture hashes;
72 of their dumps change the selected standard-builtin root set, while the
remainder change emitted-size, local-count or result-inference summaries. Five
fixtures change their recorded static result kind. The four non-ignored product
paths all remain green at `1/1`: the two Object-prototype predicates return
`boolean(true)`, heap growth returns `number(66)`, and TypedArray indexed writes
return `number(1022)`. The fifth is the existing T05 page-boundary stress test,
which remains deliberately ignored because of cost and was not run here. This
is a reviewed semantic golden delta, not a byte-identity claim.

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
functions, plus direct identifier-call `WithBaseObject`. Logical compound
assignment, destructuring/delete operations, generated class/helper contexts
and resumable captured WithObject environments remain explicit debt.

Global Object Environment identifier numeric updates now have a verified
Reference lifecycle adjacent to the already-green `with` numeric-update lane.
One private `NumericUpdateBindings` carrier fixes the old-value, result and
write-completion roles for both environment kinds. The shared binding-object
operation orders an independent GetBindingValue `HasProperty`/Get, the closed
Number-or-BigInt update, the independently rechecked SetMutableBinding, and
result publication after PutValue. A separate non-`Clone`, non-`Copy`
`GlobalObjectEnvironmentReferencePlan` owns the compiler-known global object,
name and strictness, performs the initial plain `HasProperty`, and consumes the
carrier without admitting an unscopables lookup or fallback chain. Lowering
maps all four prefix/postfix increment/decrement modes exhaustively and only
selects this plan for an unproven unresolvable global, while invalidating any
configurable global metadata to fully Dynamic runtime information.

The durable CLI oracle covers successful prefix/postfix Number and BigInt
updates, all four strict accessor-deletion modes with no recreation, sloppy
deletion and recreation, an initially absent binding throwing from GetValue
before ToNumeric, and a Proxy-prototype trace that distinguishes the initial
HasBinding, GetBindingValue recheck/Get, ToNumeric, SetMutableBinding recheck
and Set. The bounded source witness owns these exact four `noStrict` files:

- `language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue--1.js`;
- `language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue--1.js`;
- `language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue--1.js`;
- `language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue--1.js`.

The four corresponding bare-suffix `with` files and the eleven odd-suffix
global eager-compound files remain explicit regression inventories. At
pre-batch commit `f6b6af6a1779840eaf5d7c88cff2b9ff33db9381`, an isolated
current-pin run measured the prefix-increment global witness at `0/1` as
`Runtime/NotImplemented` with the exact diagnostic ``unsupported in lila
wasm-aot first slice: unbound identifier `x```; the adjacent plain-assignment
witness was `1/1`. The other three numeric files are source-proven to have
entered the same closed lowering route and refusal, but were not separately
measured pre-batch. The current integration checkpoint is green: package checks
for `lila-ir`, `lila-aot-wasm` and `lila-cli`, plus `cargo xc`, all pass; the
focused IR lifecycle test is `1/1`, four source-bounded structure executables
total `17/17`, and the Wasm lifecycle fixture is `1/1` in 45.02 seconds. The
exact selected global numeric cohort now passes `4/4`, the four bare-suffix
`with` controls remain `4/4`, and the modern eager-compound prefix remains
`22/22` with zero unsupported, crash or bug outcomes. These focused results do
not claim the complete language subtree or pinned matrix is green.

Direct identifier calls selected through a `with` Object Environment Record
now have a verified Reference capability. The private, non-copyable,
must-use `WithEnvironmentIdentifierCallReferencePlan` wraps the existing
non-empty Object Environment chain; its sole consuming `call` transition pairs
GetBindingValue's callee with the exact same binding object as
CallExpression's `WithBaseObject()` receiver. Each selected branch carries
`this_arg: Some(bindingObject)`, while the complete ordinary fallback remains
outside that path with `this_arg: None`.

The lowerer intercepts direct identifier calls before generator and
name-specific builtin folds, so a selected `Boolean` or `Number` binding cannot
bypass ResolveBinding. It locates the declarative/global fallback before the
observable HasBinding walk, forces mutable fallback value and function-target
metadata to the full runtime domain, then lowers arguments once. The AOT
consumer retains the existing callee, explicit-this, arguments and Call order.
The durable CLI oracle covers selected receiver identity, a getter deleting the
method before its retained-base call, exact `huhrgac`
HasProperty/unscopables/Get/getter/argument/call ordering, nested-unscopables
selection, strict undefined-this and sloppy global-this fallback calls,
selected builtin shadowing, an empty-with builtin fallback, and a declining
Proxy HasBinding trap which replaces the known fallback function before its
runtime GetValue.

The exact selected Test262 inventory is the single `flags: [noStrict]` file
`language/expressions/call/with-base-obj.js`. At clean pre-batch commit
`88de596ce22a69b8b7c47dacaed051172adf46b6`, it measured `0/1` under Wasm AOT
as `Bug/Runtime`, with its `via CallExpression` SameValue assertion observing
the wrong receiver. The current integration checkpoint is green: package
checks and `cargo xc` pass; the focused IR filter is `2/2`, the bounded
structure executable is `4/4`, and the exact CLI fixture is `1/1` in 23.19
seconds. The broader CLI `environment` slice is `13/13` in 315.07 seconds. One
test-only hardcoded-key expectation was corrected during the focused IR rerun.
The exact selected Test262 file now passes `1/1` with zero unsupported, crash
or bug outcomes. These focused results do not claim the complete
`language/expressions/call` or `with` subtree, or pinned matrix is green.

Object Environment identifier logical assignment now has a verified shared
Reference lifecycle for all three closed `LogicalBinaryOp::{And, Or,
Coalesce}` modes. `ObjectEnvironmentBindingObject::logical_assignment` keeps
GetBindingValue on one binding-object identity and places SetMutableBinding,
including its RHS and independent `HasProperty` recheck, wholly inside the
taken `LogicalShortCircuit` branch. The non-empty, non-copyable
`WithEnvironmentReferencePlan` wraps that lifecycle in inner-to-outer
HasProperty/unscopables selection and a pre-located fallback. The separate
non-copyable `GlobalObjectEnvironmentReferencePlan` performs one initial plain
HasProperty, emits ReferenceError on a miss, and cannot carry unscopables state.

A private, non-copyable, must-use `LocatedIdentifierLogicalAssignment` owns the
located Reference and `Option<ValueInfo>` proven-global snapshot before RHS
lowering; its consuming helper cannot silently reread metadata after the RHS
has changed it. The durable CLI oracle makes that snapshot load-bearing with an
untaken String-writing RHS beside an old Number value. It also covers all three
modes for initially absent globals, dynamically present short circuits with no
RHS or PutValue, taken global writes, strict getter deletion without Set,
sloppy recreation, visible and fallback `with` bindings, nested
`Symbol.unscopables`, and the exact observable `huhgdrhs` order from HasBinding
through SetMutableBinding.

The exact selected `onlyStrict` Test262 files are:

- `language/expressions/logical-assignment/lgcl-and-assignment-operator-unresolved-lhs.js`;
- `language/expressions/logical-assignment/lgcl-or-assignment-operator-unresolved-lhs.js`;
- `language/expressions/logical-assignment/lgcl-nullish-assignment-operator-unresolved-lhs.js`.

The current integration checkpoint is green: package checks for `lila-ir`,
`lila-aot-wasm` and `lila-cli`, plus `cargo xc`, all pass; the focused IR
lifecycle tests are `2/2`, and four final source-bounded structure executables
are `4/4`. One stale derive marker was corrected during that focused structural
rerun without changing product code. The exact Wasm lifecycle fixture is `1/1`
in 87.88 seconds, while the broader focused `environment` test selection is
`12/12` in 270.93 seconds. The three selected strict unresolved-lhs Test262
files now pass `3/3` with zero unsupported, crash or bug outcomes. The six
adjacent unresolved-RHS physical files pass all `12/12` sloppy and strict
executions. No vendored logical-assignment witness contains `with`, so the
fixture remains the honest evidence for that behavior. These focused results
do not claim the complete language subtree or pinned matrix is green.

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

`AsyncForOfArrayWalkForm` was a private, non-capability one-shot classification
at this checkpoint. Its four-mention ownership and exhaustive projection are
recorded in the retired contract
[`async-for-of-array-walk-form-ownership.md`](../docs/rust-rewrite/contracts/async-for-of-array-walk-form-ownership.md).
The later T15 resumable synchronous Iterator Record migration deleted the form,
its specialized Array lowerer, and its premise witness. That later checkpoint
does not change this source-equivalent lane's historical verification result.

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
global Object Environment cases; they are explicit follow-up evidence, not part
of this `with`-scope passing claim.

That adjacent global Object Environment follow-up is now verified around a
distinct non-copyable `GlobalObjectEnvironmentReferencePlan`. Its constructor
owns the compiler-provided global object rather than accepting an arbitrary
expression, and its consuming compound-assignment operation performs the
Global Object Record's initial plain `HasProperty` before reusing the sealed
old-value/result/write carrier and shared independent GetBindingValue and
SetMutableBinding rechecks. Unlike `WithEnvironmentReferencePlan`, this type
cannot carry an outer fallback chain or `Symbol.unscopables` selection. The
durable CLI fixture covers all eleven directly evidenced operators, an initial
miss throwing before RHS evaluation, strict accessor deletion without
recreation, sloppy accessor deletion with recreation, inherited selection and
result publication only after PutValue succeeds. The source-bounded witness
owns the exact eleven odd-suffix `noStrict` Test262 files and retains the whole
22-file modern prefix as an adjacent regression gate for the already-green
`with` siblings. At pre-batch commit
`450f67050a270eebb4459b8ebd3cb2b171f5b7ee`, that prefix reported `11/22`: all
eleven selected global executions were `Runtime/NotImplemented` with the exact
diagnostic ``unsupported in lila wasm-aot first slice: unbound identifier
`x```; all eleven `with` executions passed. The affected-package compile is
green; the IR lifecycle test is `1/1`, the new source-bounded suite is `4/4`,
the retained compound/numeric suites are `5/5` and `4/4`, and the Wasm
lifecycle fixture is `1/1`. The selected current-source Test262 cohort now
passes `11/11`, and the adjacent prefix passes `22/22` with zero unsupported,
crash or bug outcomes. The shared closed operation includes `**=`, but there is
no twelfth direct vendored witness and no full language-subtree or pinned-matrix
claim.

The declaration-instantiation token now carries its frame-lifetime decision in
the private, capability-free `InstantiatedFrame::{Pushed, Current}` domain.
The two frame-owning constructors push before sweeping and produce `Pushed`;
the current-frame constructor performs no push and produces `Current`.
Consuming `finish` matches both rows exhaustively to pop or preserve the frame,
and a bounded structure guard pins the exact three producers, sweep ordering
and both lifecycle arms. The structure target passes `3/3`, and the exact block
and switch TDZ witnesses pass `2/2`. Independent review confirmed the complete
capability/mention closure and preserved push, sweep and pop order. Coordinated
`cargo xc`, formatter, diff and repository policy checks are green. This is a
source-equivalent T08 invariant closure, not a new environment or control-flow
capability.

The shared private-property compound-assignment and super/private
logical-assignment path now hands its operation through the private,
non-derived `PropertyUpdateOp::{Arithmetic, Bitwise, Logical}` domain. Its sole
consumer uses one exhaustive match to bind RHS reachability, value operation,
shape and composition; the former copied `matches!` reachability observation
is gone. The Rust-lexical `property_update_op_ownership_structure` target pins
the nine-to-eight ownership census, one producer and consumer per row, all
three ordered producer contexts and the complete Reference read-to-write
lifecycle. It passes `4/4`; the affected-package check and neighboring logical
assignment structure are green. The four exact private-reference Test262 leaves
covering arithmetic, bitwise, taken logical and short-circuit logical behavior
pass all `8/8` sloppy/strict Wasm-AOT executions. Every failure bucket is zero.
This is source-equivalent ownership closure, not a new private-field or `super`
behavior claim.

The function arguments binding protocol now has one private, non-cloneable
pending-to-bound lifecycle. Parameter binding consumes its mapped or unmapped
construction authority once before owned arguments-object initialization;
local planning retains only a presence projection that cannot recover the
semantic protocol. The Rust-lexical
`arguments_binding_protocol_ownership_structure` target pins that closure, and
passes `4/4`; the exact repeated-binding unit passes `1/1`. The
contract and retained reusable mapped-entry projections are recorded in
[`function-arguments-binding-ownership.md`](../docs/rust-rewrite/contracts/function-arguments-binding-ownership.md).
This is a source-equivalent T08 ownership closure, not a new arguments-object
or environment behavior claim.

Prepared destructuring property keys now cross target preparation and PutValue
through the private, non-derived
`PreparedDestructuringPropertyKey::{Static, Computed}` domain. The computed row
can be constructed only after its payload and tag locals are both populated;
the write alone installs their temporary binding and exhaustively releases the
pair. The former independent key form and two `Option<u32>` fields could encode
half-prepared or contradictory states. The Rust-lexical
`prepared_destructuring_property_key_ownership_structure` target pins the
two-row domain, nine-mention authority census, ordered producer, exhaustive
projection and tag-before-payload release. The retained exact array
destructuring abrupt-completion fixture is the semantic control for the shared
prepared-property-target path. This is a source-equivalent T08 invariant
closure, not a new destructuring or control-flow capability.

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

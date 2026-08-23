# T02 — Modularize the IR and Wasm backend

**Status:** In progress — major builtin ownership bottlenecks plus the for-of and for-in lowering owner splits; broader lowering/emitter seams remain

**Parallel group:** Bootstrap/foundation  
**Depends on:** None  
**Blocks:** Safe parallel work in T04-T24

## Current repository state

Both crates now expose dedicated IR, lowering, analysis, diagnostics,
operations, ABI, heap, object, function, environment, control-flow and builtin
modules, and `./scripts/check-module-boundaries.sh` enforces the highest-value
seams and line budgets. The split remains partial: `lowering.rs`,
`builtins/standard.rs`, several family files and object/operation emitters are
still large implementation stores. Treat the landed boundaries as independent
ownership surfaces, but continue coordinating broad edits to those remaining
hotspots.

### Landed 2026-08-23: for-in lowering ownership

`lila-ir/src/lowering/for_in.rs` now owns the complete for-in lowering family:
the sole statement-facing lowerer and its twelve owner-only helpers for Annex B
initializer prefixes, closed initializer-name recovery, known-empty and
non-enumerable Test262 guards, pattern/property body prefixes, target
classification and final statement construction. The moved family was 564
source lines across four parent blocks; removing their separator lines reduced
`lowering.rs` from 22,444 to 21,877 raw lines and produced a
571-line child. Only `lower_for_in_loop` crosses the private child boundary
as `pub(super)`; all twelve helpers remain private. No IR type, public Rust API
or compiler behavior moved with it.

The statement dispatcher remains the only external caller. The parent retains
the shared for-in/of environment and TDZ lowering, loop-body and declarator
lowering, global-target recognition, single-statement and identifier matching,
static-string analysis, async-entry-state query, expression/pattern/property
lowering, scope/allocation/flow helpers, and the shared `contains`,
`supported_bound_names` and `for_in_loop_binding_storage_name` free functions.
`lowering/for_of.rs` continues to consume the shared environment helper, while
`lowering/call_expression.rs` continues to consume `for_in_global_target`; the
extraction copies neither owner and does not widen their visibility.

The byte-exact move preserves the source order of every observable decision:
refusing an awaiting body before lowering the head; retaining the Annex B
initializer prefix on every supported early exit; evaluating a nullish head
for effects; pattern-head TDZ scope push/pop; inferred-undefined parameter
widening; dynamic/array/string/object/nullish/primitive target classification;
access and pattern assignment prefixes; per-iteration lexical-environment
storage; body scope teardown before var/global flow joins; and the final
dynamic, array, string and object statement choice. The existing known-empty
and non-enumerable shortcuts, diagnostics and specialization rules are copied
exactly, not cleaned up or expanded in this batch.

The module audit requires the exact private child declaration, all thirteen
sole method owners, exactly one Rust-visible child item, zero parent copies,
zero copied shared-helper definitions, no legacy `include!` assembly and
measured parent/child line budgets. Negative controls reject a missing child,
widened module or helper, copied shared or generic helper, copied type and an
extra modifier-qualified function. Existing focused IR and CLI behavior
witnesses remain the semantic contract; no new permanent test is justified
solely by the filename move.

At clean parent `fcb0a924b`, the fresh capped pre-move Wasm golden passes `2/2`,
records 633 fixtures in 635 artifacts and is retained under
`target/golden/for-in-before-fcb0a924b`. The capped post-move golden also passes
`2/2` over the same 633 fixtures and 635 artifacts, and the recursive artifact
diff is empty. `cargo check -p lila-ir --all-targets` and `cargo xc` pass; the
focused IR witnesses pass `8/8`, `2/2`, `1/1` and `1/1`. The CLI `for_in_`
filter is unchanged at `4/7` on both the clean parent and moved tree: array
DefineProperty order, simple-object order and prototype order remain
pre-existing failures. Supplemental exact CLI witnesses are likewise unchanged
at `2/3`, with the object-keys primitive witness the pre-existing failure. This
is architecture and no-regression evidence, not a for-in behavior or
conformance improvement, broad Test262 run or full-workspace claim.

### Landed 2026-08-23: for-of lowering ownership

`lila-ir/src/lowering/for_of.rs` now owns the complete for-of lowering family:
the async array-index resumable specialization, the sole statement-facing
wrapper, the exhaustive head lowering, and the two lowering-only carriers
`AsyncForOfArrayWalkForm` and `ForOfLoweringIr`. The extraction moves one
coherent 1,026-line source family out of `lowering.rs`, `lowering_helpers.rs`
and `ir.rs`. Only `lower_for_of_loop` crosses the child boundary as
`pub(super)`; the other methods, both carriers and their
constructors/conversions are private to the child.

Moving `ForOfLoweringIr` deliberately removes an accidental public Rust API
created by `pub use ir::*`. It has no workspace consumer outside this family,
and its privacy is the invariant: every path out of `lower_for_of_head` must
construct a protocol witness, while no unrelated code may construct, retain or
discard that lowering-only proof. This is an intentional pre-1.0 API narrowing,
not a claim that the patch is only a filename change.

The statement dispatcher remains the sole external caller. Shared loop and
environment helpers stay in the parent, including `plain_async_entry_state`,
`split_resumable_loop_body`, `lower_loop_body`,
`lower_for_in_of_environment` and `lower_for_head_expression_with_tdz`.
`lowering/async_disposable.rs` retains `LoweredForOfHeadKind` plus admission,
pending-head, finalization and statement construction. Public statement/head,
environment, iterator-plan and protocol-witness IR remain in their existing
IR and iterator-obligation owners. The extraction copies none of those helpers
and does not widen their visibility.

The move preserves source order and behavior: scope push/pop on every bailout,
flow-fact snapshots and joins, iterable-before-body evaluation, synchronous-
using TDZ/disposal ordering, async-disposable pending-head sequencing,
suspension-state acquisition, five load-bearing async iterator slot
allocations, the closed array/string/generic/resumable protocol outcomes, and
`AsyncForOfArrayWalkForm`'s non-array / target-shape / captured-iteration /
captured-TDZ classification priority. No specialization cleanup, diagnostic
change, intactness repair or conformance expansion belongs in this batch.

The extraction reduces `lowering.rs` from 23,208 to 22,444 raw lines; the child
is 1,036 lines. The module audit requires the one child owner, all three sole
methods, both private carriers, zero parent/helper/IR copies, no shared-helper
copies, no legacy `include!` assembly and separate parent/child budgets. Its
missing-child, public-module, public-carrier, copied-shared-helper and generic-
helper negative controls all fail correctly. The two source-bounded for-of
backend tests read the new child directly.

The capped pre/post Wasm goldens both pass `2/2`, record 633 fixtures in 635
artifacts and have an empty recursive diff; the post capture finished in
388.51 seconds. The focused IR witnesses pass `3/3`, `3/3` and `1/1`; the three
backend structure targets pass `5/5`, `5/5` and `3/3`; and four exact CLI
witnesses pass `4/4`. `cargo check -p lila-ir --all-targets`, `cargo xc`,
formatting, diff, module-boundary and task-plan checks are green. No broad
Test262 filter or full workspace test was used as evidence. No for-of behavior
or conformance improvement is claimed.

### Landed 2026-08-23: with-statement ownership

`lila-ir/src/lowering/with_statement.rs` now owns the complete Object
Environment lifecycle for `with`: outer-environment object evaluation,
analyzed hidden-binding materialization, resumable/capture refusal, ordered
environment-chain entry and exit, body lowering and nested lexical-block IR
assembly. The statement dispatcher remains its sole caller; allocation,
suspension and reference-lifecycle helpers remain in their shared owners.

This is an exact source move. All 99 method lines compare exactly after
normalizing only `fn lower_with_statement` to private-module visibility. The
extraction reduces `lowering.rs` from 23,307 to 23,208 raw lines, and the child
is 103 lines. The module audit requires the sole owner, rejects copied shared
helpers and lifecycle types, requires exactly one chain entry and exit, forbids
legacy `include!` assembly, budgets parent and child separately, and fails both
missing-child and missing-exit negative controls. No persistent structural test
used the old method as a source-slice sentinel.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Seven targeted IR environment, definition-
cursor, hidden-capture, suspension and fallback witnesses pass `7/7`. The five
targeted CLI fixtures pass `2/5`; one combined run at untouched parent
`870203481` reproduces the other three failures with identical diagnostics:
two existing `instanceof`-callability errors and one Wasmtime function-size
limit. A deliberately broader `with_` IR filter passed `66/67`; its unrelated
Object.seal result-shape assertion remains red and was not used as evidence for
this owner. The all-target `lila-ir` and workspace checks are green. No `with`
behavior or conformance improvement is claimed.

### Landed 2026-08-23: break/continue ownership

`lila-ir/src/lowering/break_continue.rs` now owns labelled and unlabelled
break/continue target validation and final abrupt-control IR assembly. The
statement dispatcher remains its sole caller. The parent-owned active-label
stack and `LabelTargetKind` stay shared with labelled-statement lowering, while
the breakable and loop depths remain lowerer state.

This is an exact source move. All 41 method lines compare exactly after
normalizing only `fn lower_break` and `fn lower_continue` to private-module
visibility. The extraction reduces `lowering.rs` from 23,348 to 23,307 raw
lines, and the child is 45 lines. The module audit requires both sole owners,
rejects copies of the shared active-label types, forbids legacy `include!`
assembly, budgets parent and child separately, and fails its missing-child
negative control. No persistent structural test used either old method as a
source-slice sentinel.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Three focused IR loop, switch-label and
direct-label-target witnesses pass `3/3`; five focused CLI inspect, finally,
iterator-closing and using-loop lifecycle witnesses pass `5/5`. The all-target
`lila-ir` and workspace checks are green. No break/continue behavior or
conformance improvement is claimed.

### Landed 2026-08-23: switch-statement ownership

`lila-ir/src/lowering/switch_statement.rs` now owns discriminant and selector
evaluation, the one shared CaseBlock lexical environment, hoisted and Annex B
function selection, case-body lowering, flow-fact joins, breakable-depth
lifecycle and final `Switch` IR assembly. The statement dispatcher remains its
sole caller; reusable statement-list, function, environment and flow helpers
remain parent-owned.

This is an exact source move. All 86 method lines compare exactly after
normalizing only `fn lower_switch` to private-module visibility. The extraction
reduces `lowering.rs` from 23,434 to 23,348 raw lines, and the child is 90
lines. The module audit requires the sole owner, rejects copied shared
statement-list and environment-materialization helpers, forbids legacy
`include!` assembly, budgets parent and child separately, and fails its
missing-child negative control. Two for-of structural tests now end their
source slice at the adjacent `lower_for_init` owner instead of depending on the
old location of `lower_switch`.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Six focused IR CaseBlock, TDZ, Annex B,
capture and label witnesses pass `6/6`; two focused CLI inspect and throwing
property-read witnesses pass `2/2`; the two affected for-of structural targets
pass `5/5` each. The all-target `lila-ir` and workspace checks are green. No
switch behavior or conformance improvement is claimed.

### Landed 2026-08-23: labelled-statement ownership

`lila-ir/src/lowering/labelled_statement.rs` now owns nested-label collection,
direct labelled-function routing, loop-versus-breakable target classification,
active-label stack installation/removal and final `Labelled` IR assembly. The
statement dispatcher remains its sole caller; the shared `ActiveLabel` and
`LabelTargetKind` types remain parent-owned for break/continue lowering.

This is an exact source move. All 68 method/helper lines compare exactly after
normalizing only `fn lower_labelled` to private-module visibility. The
extraction reduces `lowering.rs` from 23,502 to 23,434 raw lines, and the child
is 72 lines. The module audit requires the sole owner and both private helpers,
rejects copies of the shared label types, forbids legacy `include!` assembly,
budgets parent and child separately, and fails its negative control when the
child is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Five focused IR label/target/lifecycle filters
pass `5/5`; three focused CLI inspect, iterator-closing and await-using filters
pass `3/3`. The all-target `lila-ir` and workspace checks are green. No labelled
statement behavior or conformance improvement is claimed.

### Landed 2026-08-23: while-family ownership

`lila-ir/src/lowering/while_loop.rs` now owns ordinary and resumable `while`
lowering plus the explicit `do while` suspension refusal. Condition/body order,
loop flow-fact joins, async/generator resume-state selection and the final
`While`, `DoWhile` or `GeneratorLoop` choice move together. The statement
dispatcher remains their sole caller; shared loop-resumption helpers remain in
the parent.

This is an exact source move. All 99 source lines compare exactly after
normalizing the two private-module visibility tokens and rustfmt's wrapped
`lower_do_while_loop` signature. The extraction reduces `lowering.rs` from
23,601 to 23,502 raw lines, and the child is 106 lines. The module audit
requires both sole owners, rejects copies of `plain_async_entry_state` and
`split_resumable_loop_body`, forbids legacy `include!` assembly, budgets parent
and child separately, and fails its negative control when the child is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Five focused IR loop/resumption/refusal
filters pass `5/5`; three focused CLI lexical-environment, abrupt-finally and
iterator flat-map filters pass `3/3`. The all-target `lila-ir` and workspace
checks are green. No while/do-while behavior or conformance improvement is
claimed.

### Landed 2026-08-23: if-statement ownership

`lila-ir/src/lowering/if_statement.rs` now owns the complete conditional
lifecycle: condition lowering and static selection, branch-local var/global
facts, post-branch joins, abrupt-completion result typing and generator
yield-state splitting/merging. The statement dispatcher remains its sole
caller; shared static-expression helpers remain in the parent.

This is an exact source move. All 137 method/helper lines compare exactly after
normalizing only `fn` to `pub(super) fn` on the owner method. The extraction
reduces `lowering.rs` from 23,738 to 23,601 raw lines, and the child is 141
lines. The module audit requires the sole owner and both private lifecycle
helpers, rejects a copied `static_bool_expr`, forbids legacy `include!`
assembly, budgets parent and child separately, and fails its negative control
when the child is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Six focused IR branch/flow/resumption filters
pass `6/6`; four focused CLI inspect, Wasm, async-generator and finally filters
pass `4/4`. The all-target `lila-ir` and workspace checks are green. No
if-statement behavior or conformance improvement is claimed.

### Landed 2026-08-23: classic-for ownership

`lila-ir/src/lowering/for_loop.rs` now owns the complete classic `for`
lifecycle: async-disposable head validation, resumable-loop eligibility,
lexical TDZ setup, initializer and per-iteration Environment Record selection,
test/update/body lowering, flow-fact merging, suspension-state construction and
the final `For` or `GeneratorLoop` IR choice. The statement dispatcher remains
its sole caller; reusable loop, scope, flow and resumption helpers remain in the
parent.

This is an exact source move. The only method-body change is private-module
visibility from `fn` to `pub(super) fn`; all 209 source lines compare exactly
after normalizing that token. The extraction reduces `lowering.rs` from 23,947
to 23,738 raw lines, and the child is 213 lines. The module audit requires the
sole owner, forbids a second parent body or legacy `include!` assembly, budgets
the parent and child separately, and fails its negative control when the child
is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Eight focused IR lifecycle/refusal filters
pass `8/8`; three focused CLI lexical-environment, async-disposable and
throw-propagation filters pass `3/3`. No classic-for behavior or conformance
improvement is claimed.

### Landed 2026-08-23: statement-dispatch ownership

`lila-ir/src/lowering/statement.rs` now owns the exhaustive `Statement`
dispatcher and its resumable expression-statement specialization. Direct and
nested async `await`, generator `yield` and assignment resumption, staged
generator templates/expressions, ordinary expression statements and every
control-flow/declaration delegate remain in one closed dispatch. The focused
statement implementations and reusable suspension helpers remain parent-owned.

This is an exact source move. The only method-body change is private-module
visibility from `fn` to `pub(super) fn`; all 255 source lines compare exactly
after normalizing that token. The extraction reduces `lowering.rs` from 24,202
to 23,947 raw lines, and the child is 259 lines. The module audit requires the
sole owner, forbids a second parent body or legacy `include!` assembly, budgets
the parent and child separately, and fails its negative control when the child
is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Seven focused IR statement/resumption
filters pass `7/7`; four focused CLI inspect and Wasm control-flow filters pass
`4/4`. No statement behavior or conformance improvement is claimed.

### Landed 2026-08-23: new-expression ownership

`lila-ir/src/lowering/new_expression.rs` now owns the complete `lower_new`
lifecycle: constructor target resolution, spread-aware argument evaluation,
builtin and user-constructor result typing, Proxy trap-hint observation,
dynamic-source rejection, instance-prototype inference and static RegExp
compilation. The parent expression dispatcher remains its sole caller.

This is an exact source move. The only source-body change is private-module
visibility from `fn` to `pub(super) fn`; all constructor branches and emitted
IR choices compare exactly. The extraction reduces `lowering.rs` from 24,446
to 24,202 raw lines; the formatted child is 248 lines. The module audit
requires the sole owner, forbids a second parent body or legacy `include!`
assembly, and budgets parent and child separately.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. The all-target `lila-ir` and workspace checks
are green. Five focused IR filters pass `2/2`, `1/1`, `1/1`, `1/1` and `1/1`;
the Map/Set iterable-construction filter fails `0/2` both here and at untouched
parent `394e8fda7` with the same shape assertions. Five focused CLI filters
pass `1/1`, `2/2`, `1/1`, `1/1` and `1/1`. No constructor behavior or
conformance improvement is claimed.

### Landed 2026-08-23: property-access ownership

`lila-ir/src/lowering/property_access.rs` now owns the complete ordinary,
private and super property-access dispatcher. Primitive auto-boxing, array and
arguments exotic routing, well-known Symbol recognition, property-hook
observation and unknown-effect invalidation move together; the parent
expression dispatcher remains the sole caller.

The target-kind match now names `ValueKind::Number` explicitly instead of
using a catch-all for its existing unsupported result. That preserves current
behavior while making a future `ValueKind` addition a compile error until this
dispatcher assigns it semantics. The module audit requires that exhaustive
arm, forbids the old catch-all and enforces single ownership.

The source body is otherwise exact after normalizing private-module
visibility. The extraction reduces `lowering.rs` from 24,663 to 24,446 raw
lines; the formatted child is 223 lines. The capped pre/post Wasm goldens both
pass `2/2`, capture 635 artifacts each and have an empty recursive diff. The
all-target `lila-ir` and workspace checks are green. Serial IR filters pass
`2/2` for `property_access`, `6/6` for `property_read`, `1/1` each for
`symbol_description` and `dynamic_string_property`, and `34/34` for `call_`;
the corresponding focused CLI filters pass `3/3`, `1/1` and `6/6`. No
property-access behavior or conformance improvement is claimed.

### Landed 2026-08-23: try-statement ownership

`lila-ir/src/lowering/try_statement.rs` now owns the complete `lower_try`
lifecycle: try/catch/finally block lowering, catch-parameter Environment Record
construction, thrown-value inference, generator/async resume planning and final
`TryCatch`, `TryFinally` or `TryCatchFinally` assembly. The parent statement
dispatcher remains its sole caller, while reusable block and throw-analysis
helpers remain parent-owned.

The former eight-field catch tuple and five-field finally tuple are now private
`LoweredCatchClause` and `LoweredFinallyClause` records. Named generator/async
entry and exit fields make positional state transposition impossible at every
plan and final-assembly use site. The module audit requires both records and
rejects positional tuple-field access in this owner.

The extraction reduces `lowering.rs` from 24,910 to 24,663 raw lines; the
formatted child is 264 lines. The capped pre/post Wasm goldens both pass `2/2`,
capture 635 artifacts each and have an empty recursive diff. The focused
all-target `lila-ir` check and the new module boundary are green. Serial IR
filters pass `12/12` for `try_` and `14/14` for `catch`; the CLI `finally`,
`catchability` and `top_level_try` filters pass `2/2` each. No try-statement
behavior or conformance improvement is claimed.

### Landed 2026-08-23: delete-expression ownership

`lila-ir/src/lowering/delete_expression.rs` now owns the complete
`lower_delete` target dispatcher across ordinary/private/super property
References, identifier deletion and non-Reference values. The parent retains
the sole unary-expression call and the reusable shape, strictness and property
helpers consumed by the implementation.

This is a semantic-free source move. All 213 method lines compare exactly to
the pre-move implementation after normalizing only `fn` to `pub(super) fn`.
The private child boundary reduces `lowering.rs` from 25,123 to 24,910 raw
lines; the formatted child is 217 lines. The module audit requires the sole
owner method, forbids a second parent body or legacy `include!` assembly, and
budgets parent and child separately.

The extraction does not combine the two currently duplicate computed-key
branches. That cleanup can be reviewed separately after this exact move's
behavioral checkpoint; this commit preserves the existing instruction and
invalidation choices byte-for-byte at the Rust source level. No delete behavior
or conformance improvement is claimed.

The capped workspace/all-target check is green. Serial delete-focused coverage
passes `7/7` in `lila-cli`, `2/2` in `lila-aot-wasm` and `4/4` in
`lila-engine`. Formatting, exact source comparison, module-boundary and
task-plan audits are green.

### Landed 2026-08-23: assignment-expression ownership

`lila-ir/src/lowering/assignment.rs` now owns the complete exhaustive
`lower_assign` dispatcher across identifier, property, private, destructuring,
logical and eager compound assignment. The parent expression match remains its
single caller. Specialized ordinary-property and Object Environment Record
Reference lifecycles remain in their existing typed child modules.

This is a semantic-free source move. All 706 body lines compare exactly to the
pre-move implementation; the signature changes only private-module visibility
and rustfmt's multiline layout. The child boundary reduces `lowering.rs` from
25,830 to 25,123 raw lines; the formatted child is 716 lines. The module audit
requires the sole owner method, forbids a second parent body or legacy
`include!` assembly, and budgets parent and child separately.

The capped workspace/all-target check and serial IR `assignment` cohort
(`34/34`) are green. The serial CLI `assignment` cohort reports `6/7` both
before and after extraction; the same with-environment compound-assignment
fixture fails with the same completion and error text. Formatting, exact body
comparison, module-boundary and task-plan audits are green. No assignment
behavior or conformance improvement is claimed.

### Landed 2026-08-23: ordinary function-definition ownership

`lila-ir/src/lowering/function_definition.rs` now owns the complete ordinary
`lower_function` lifecycle: nested lowerer creation, analysis-state transfer,
parameter and body lowering, capture/lexical-environment planning, signature
updates, resumable metadata and final `FunctionIr` construction. The seven
top-level orchestration calls remain in the parent; parameter helpers shared
with generated iterators, class methods and object methods also remain there.

This is a semantic-free source move. All 717 method lines compare exactly to
the pre-move implementation after normalizing only `fn` to `pub(super) fn`.
The private child boundary reduces `lowering.rs` from 26,547 to 25,830 raw
lines; the formatted child is 721 lines. The module audit requires the sole
owner method, forbids a second parent body or legacy `include!` assembly, and
budgets parent and child separately.

The capped workspace/all-target check and serial IR `function_` cohort
(`61/61`) are green. The serial CLI `functions::` cohort reports `45/49`; both
inspect-shape assertions and both mapped-arguments semantics fixtures reproduce
at the exact pre-extraction commit `bda775dfc`. Formatting, exact source
comparison, module-boundary and task-plan audits are green. No function
behavior or conformance improvement is claimed.

### Landed 2026-08-23: builtin call-result analysis ownership

`lila-ir/src/lowering/builtin_call_info.rs` now owns the complete exhaustive
`StandardBuiltinId` result analysis: return kinds and shapes, boxed-builtin
accounting, callback parameter observations and the few result-dependent flow
invalidations. Construct lowering, general resolved calls, RegExp literal
lowering and well-known-symbol method routing remain its four consumers.

This is a semantic-free source move. All 2,146 method lines compare exactly to
the pre-move implementation after normalizing only `fn` to `pub(super) fn`.
The private child boundary reduces `lowering.rs` from 28,693 to 26,547 raw
lines; the formatted child is 2,150 lines. The module audit requires the sole
owner method, forbids a second parent body or legacy `include!` assembly, and
budgets parent and child separately.

The capped workspace/all-target check and the serial CLI `call_` cohort (`6/6`)
are green. The current serial IR `call_` cohort is also green (`34/34`) after a
follow-up contract refresh accepted both typed `PropertyRead` and canonical
`GetV` as the same materialized method Reference read. Formatting, exact source
comparison, module-boundary and task-plan audits are green. No call behavior or
conformance improvement is claimed.

### Landed 2026-08-23: Atomics backend ownership

`lila-aot-wasm/src/builtins/atomics.rs` now owns all fourteen Atomics builtin
bodies, the shared integer-operation machinery, synchronous and asynchronous
wait/notify state transitions, host-agent calls, and atomic memory access
helpers. The flat catalog dispatch retains one typed delegate per builtin
through the closed `AtomicsBuiltin` domain; the family file cannot accept an
unrelated `StandardBuiltinId`.

The extraction also replaces the old RMW helper's broad nine-case operation
parameter and four `unreachable!` catch-alls with a six-case
`AtomicsRmwOperation`. Load, store and compare-exchange can no longer reach an
RMW opcode selector. Only three methods cross the family boundary: the BigInt
element-kind predicate shared with `TypedArray.prototype.with`, the event-loop
waiter-drain checkpoint, and the Promise-job waiter-poll checkpoint.

The moved emitter bodies and their instruction sequences are source-identical
to the pre-move implementation. The non-emitting structural changes are the
typed family dispatch, the narrower RMW carrier and one deliberate visibility
change. The `Atomics.isLockFree` body is unchanged apart from becoming a
private family method. `standard.rs` falls from 33,275 to 30,567 raw lines;
the formatted family file is 2,805 lines. The boundary audit requires the
module, all fourteen typed delegates, the three closed domains, the three
reviewed cross-family hooks, exhaustive matches and separate line budgets for
parent and child.

The capped workspace/all-target check is green. Focused serial Atomics coverage
passes `2/2` in `lila-aot-wasm` and `5/5` in `lila-engine`; the CLI cohort passes
`12/13`. Its remaining `Atomics.isLockFree` core-fixture failure reproduces at
the untouched parent commit, so this structural extraction neither owns nor
claims to fix it. Formatting, module-boundary and task-plan audits are green.

### Landed 2026-08-23: call-expression lowering ownership

`lila-ir/src/lowering/call_expression.rs` now owns the complete `lower_call`
implementation, including direct-call recognition, builtin and method routing,
argument evaluation, specialization and final indirect-call construction. The
parent keeps expression dispatch and the reusable call, shape and static-value
helpers consumed by that implementation.

This is a semantic-free source move. The only visibility change is the one
`pub(super)` method consumed by parent expression dispatch; the child remains
inside the private `lowering` module. The extraction reduces `lowering.rs` from
31,833 to 28,693 raw lines. The boundary audit requires the child module and
sole owner method, forbids a second parent body or legacy `include!` assembly,
budgets the child at 3,200 raw lines and tightens the parent budget to 29,000.

The extraction is verified by an exact normalized source-body comparison with
the pre-move implementation and a green `cargo check --workspace --all-targets`.
The serial call-focused IR cohort reports 63/67 and the serial CLI call cohort
reports 35/36. All five failures reproduce unchanged at pre-extraction commit
`f2309be48`: four primitive/ordinary method-call inference contracts and the
`arguments.callee` CLI fixture. The module-boundary and task-plan audits are
green. No call behavior or conformance improvement is claimed.

A later contract refresh removed the last current IR `call_` false negative.
Flow widening legitimately selects the canonical `GetV` carrier for an
ordinary method read; the test now verifies that either `GetV` or the typed
`PropertyRead` consumes the one materialized receiver before Call supplies the
same value as `this`. The focused test and current serial IR cohort pass `1/1`
and `34/34`; an independent Wasm-AOT witness completes with `boolean(true)`.
Production lowering and emitted semantics are unchanged.

### Landed 2026-08-23: class-definition lowering ownership

`lila-ir/src/lowering/class_definition.rs` now owns the complete
`lower_class_common_in_name_scope` implementation: heritage validation, public
and private method/field planning, auto-accessor backing and generated-function
scheduling, instance/static initialization plans, shapes and final typed
`ClassDefinitionIr` construction. The parent keeps the declaration/expression
entrypoints, class-name scope orchestration and generated-function helpers.

This is a semantic-free source move. The only visibility change is the one
`pub(super)` method consumed by the parent orchestrator; the child remains
inside the private `lowering` module. The extraction reduces `lowering.rs` from
33,156 to 31,833 raw lines, restoring the enforced 32,000-line boundary. The
boundary audit requires the child module and sole owner method, forbids a
second parent body or legacy `include!` assembly, and budgets the child at
1,400 raw lines.

The extraction is verified by an exact normalized source-body comparison with
the pre-move implementation, `cargo check --workspace --all-targets`, the
focused IR auto-accessor regression (1/1), and the serial CLI class group
(27/27). The module-boundary and task-plan audits are also green. No class
behavior or conformance improvement is claimed.

### Landed 2026-08-17: bound-function invoker ownership

The hidden `BoundFunctionInvoker` body now lives beside
`Function.prototype.bind` in `builtins/function.rs`. `FunctionBuiltin` is the
closed six-case backend domain for the five public Function intrinsics and the
one internal call/construct invoker they create; `builtins/standard.rs` keeps
only one typed delegate per case. This closes the remaining Function-owned body
that unrelated builtin work still had to cross in the shared dispatcher and
reduces that parent from 33,342 to 33,248 raw lines.

The invoker body is a semantic-free source move. Comparing it with the frozen
pre-move `builtins/standard.rs` arm after normalizing only the enum qualifier
shows the same instruction, temporary-local reservation and reverse-release
order. Its existing call, construct, nested `new.target` and heap-rooting CLI
fixtures remain the behavioral characterization; they were not executed while
Cargo and runtime verification remain under the centralized lease.

The module-boundary audit requires all six typed delegates, the unique hidden
variant and its unique child match arm, rejects catch-all/unreachable escape
routes, and budgets both the newly complete family file and the smaller parent.
The static write-phase gates are green: `git diff --check`, the bounded
source-body comparison, `check-module-boundaries.sh`, `check-task-plan.sh` and
focused `rustfmt`. No Function, bind, call/construct, `new.target`, heap-rooting
or conformance behavior improvement is claimed by this extraction.

### Landed 2026-08-13: Number intrinsic ownership

The complete eleven-member Number intrinsic family now lives in
`builtins/number.rs`: `Number`, its four non-coercing predicates and its six
prototype methods. The catalog match keeps one typed delegate per ID. A closed
`NumberBuiltin` owns the parent-facing family choice, and a private closed
`NumberPrototypeOperation` owns the six operations that share receiver
validation. Neither domain admits an unrelated `StandardBuiltinId`; both
behavior matches are exhaustive and the boundary audit rejects catch-all arms.

This is a semantic-free source move. Static source/token comparisons against
the parent commit `4dec427e6` confirm the same emitted instruction and temporary-
local order for all four predicates, the Number and remaining String constructor
paths, the shared prototype receiver path, all six prototype operations and the
final local releases. Numeric conversion and formatting algorithms remain with
`operations.rs`. A focused CLI fixture characterizes the intended unchanged
runtime family surface, but has not executed while the resource-bounded matrix
owns Cargo and Test262.

The static write-phase gates are green: `git diff --check`,
`check-module-boundaries.sh`, `check-task-plan.sh` and
`rustfmt --edition 2024 --check --config skip_children=true` over the four
touched Rust files (`builtins/mod.rs`, `builtins/number.rs`,
`builtins/standard.rs` and `cli/language_numerics.rs`). `skip_children=true` is
required so this focused gate does not recursively format unrelated builtin
modules. Compile, focused fixture execution, current-pin Number and pre/post
golden gates remain deferred. The pre-edit golden must later be built from
parent commit `4dec427e6` in a separate worktree after copying the committed
`wasm_number_builtin_family.js` fixture into that worktree. The before and after
captures therefore use the same 583-fixture corpus and their formal `diff -r`
remains meaningful and reproducible. No Number behavior or conformance
improvement is claimed by this extraction.

### Landed 2026-08-12–13: builtin metadata and family body boundaries

Fourteen previously coupled builtin stores now have separate owners:

- `lila-ir/src/lowering/builtin_shapes.rs` owns 98 pure shape/signature
  constructors. At extraction, `lowering.rs` fell from 39,177 to 31,979 lines;
  subsequent work leaves it at 31,998 lines, below the enforced cap. The moved
  methods have only parent-module visibility except the existing crate test
  hook.
- `lila-ir/src/builtins/catalog.rs` is the single 779-row
  `StandardBuiltinId` registry. One row generates the enum, names, flags,
  function-ID mappings and independent function/global order arrays. Typed
  dense ordinals plus const duplicate/hole/ID checks preserve the deliberately
  different declaration, 779-function and 52-global orders.
- `lila-aot-wasm/src/builtins/object.rs` owns all 34 Object builtin bodies and
  their private helpers. Three grouped choices are closed enums rather than
  generic builtin IDs or booleans.
- `lila-aot-wasm/src/builtins/proxy.rs` owns the three Proxy lifecycle bodies;
  `reflect.rs` remains the separate owner of the 13 Reflect bodies and their
  proxy-trap machinery.
- `lila-aot-wasm/src/builtins/math.rs` owns all 37 Math bodies behind a private
  closed `MathBuiltin` domain. `standard.rs` gives every Math ID its own typed
  one-line delegate, and both Math behavior matches are exhaustive. The
  min/max direction is a two-case private enum rather than a generic builtin
  ID. After the Object, Proxy and Math moves, `standard.rs` has fallen from
  49,179 to 36,807 lines.
- `lila-aot-wasm/src/builtins/symbol.rs` owns all seven Symbol bodies and the
  three shared Symbol receiver/description helpers behind a private closed
  `SymbolBuiltin` domain. `String(symbol)` reaches the one helper it shares
  through a parent-private method; the remaining helpers cannot escape the
  family. The catalog dispatch keeps seven typed delegates, and `standard.rs`
  fell from 36,789 to 36,313 lines without changing an emitted instruction.
- `lila-aot-wasm/src/builtins/bigint.rs` owns all six BigInt intrinsic bodies
  behind a private closed `BigIntBuiltin` domain. The constructor, signed and
  unsigned fixed-width operations, and three prototype methods moved verbatim;
  general BigInt conversion, allocation and stringification helpers remain
  with their existing operation and heap owners. The catalog dispatch keeps
  six typed delegates, and `standard.rs` fell from 36,313 to 35,647 lines.
- `lila-aot-wasm/src/builtins/boolean.rs` owns all three Boolean intrinsic
  bodies behind a private closed `BooleanBuiltin` domain. The constructor keeps
  the same argument/result-local ordering previously shared with Number and
  String, while the two prototype methods keep their boxed-receiver checks and
  realm-local TypeError route together. After the intervening T20 residue
  consolidation, the extraction reduced `standard.rs` from 35,532 to 35,439
  lines.
- `lila-aot-wasm/src/builtins/number.rs` owns all eleven Number intrinsic bodies
  behind a closed `NumberBuiltin` domain. A private closed
  `NumberPrototypeOperation` owns the six methods that share Number receiver
  validation; the constructor and four static predicates complete the family.
  Eleven typed delegates preserve the flat catalog dispatch, and `standard.rs`
  fell from 33,730 to 33,512 lines.
- `lila-aot-wasm/src/builtins/function.rs` owns the complete five-member
  Function intrinsic family and its hidden bound-function call/construct
  invoker behind a private closed `FunctionBuiltin` domain. The catalog
  dispatch keeps six typed delegates, while the moved bodies retain their exact
  instruction and temporary-local order. The original intrinsic extraction
  reduced `standard.rs` from 34,461 to 34,088 lines; moving the remaining
  invoker body reduces it again without changing the public family.
- `lila-aot-wasm/src/builtins/uri.rs` owns all six global URI and Annex-B codec
  wrappers behind a private closed `UriBuiltin` domain. The UTF-8/UTF-16 codec
  primitives remain with their existing `string.rs` owner; only the complete
  global wrapper family moved. Six typed delegates preserve the flat catalog
  dispatch, and `standard.rs` fell from 35,439 to 35,394 lines.
- `lila-aot-wasm/src/builtins/global_numeric.rs` owns both coercing global
  numeric predicate bodies behind a private closed `GlobalNumericBuiltin`
  domain. `Number.isFinite` and `Number.isNaN` remain with the distinct
  non-coercing Number family, while `parseInt` and `parseFloat` remain host
  builtin emitters. The catalog dispatch keeps one typed delegate for each of
  `isFinite` and `isNaN`, and `standard.rs` fell from 35,394 to 35,372 lines.
- `lila-aot-wasm/src/builtins/errors.rs` owns the complete eleven-member Error
  intrinsic family as well as its pre-existing allocation, realm-prototype,
  cause, iterable and throw helpers. A private closed `ErrorBuiltin` domain
  distinguishes the static predicate, the nine constructors carried by the
  existing closed `NativeErrorKind`, and `Error.prototype.toString`; unrelated
  `StandardBuiltinId` values cannot reach this family emitter. Eleven typed
  delegates preserve the catalog dispatch without duplicating the error-kind
  registry, and `standard.rs` fell from 35,372 to 34,948 lines.
- `lila-aot-wasm/src/builtins/json.rs` owns all four JSON namespace bodies
  alongside the parse, reviver, stringify and raw-JSON machinery they already
  consume. A private closed `JsonBuiltin` domain covers `parse`, `stringify`,
  `rawJSON` and `isRawJSON`; hidden static-JSON lowering and runtime helpers
  remain implementation details rather than pretend namespace members. Four
  typed delegates preserve the flat catalog dispatch, and `standard.rs` fell
  from 34,948 to 34,461 lines.
- `lila-aot-wasm/src/builtins/atomics.rs` owns all fourteen Atomics bodies,
  integer/RMW operation domains, wait queues, host-agent calls and atomic
  memory helpers. Fourteen typed delegates preserve the catalog dispatch;
  three explicitly checked helpers remain visible to the TypedArray,
  event-loop and Promise consumers. `standard.rs` fell from 33,275 to 30,567
  lines.

The earlier central feature-enabled CLI compile, which covers `lila-aot-wasm`
and `lila-intl`, and the focused builtin catalog tests pass for the moves that
reached that checkpoint. The source moves were also compared against their
pre-extraction bodies, and the boundary audit prevents these stores from being
folded back into their parents. The later
Proxy move is source-equivalent by a static body comparison and is included in
the green compile checkpoint and product-artifact boundary proof. The Math move
is statically source-equivalent, boundary-checked, and covered by that compile
checkpoint. The later Symbol move is statically source-equivalent and
boundary-checked, passes the centralized feature-enabled CLI compile, and is
covered by the exact String/Symbol hook fixture through the product Wasm
backend. The BigInt move is statically source-equivalent and boundary-checked;
its centralized feature-enabled compile and the exact constructor/fixed-width,
wrapper-coercion and cross-realm prototype behavior checkpoints are green.
The Boolean move is statically instruction-sequence equivalent and
boundary-checked; its compile, focused fixture, and real Boolean shard gates
remain queued behind the active resource-bounded matrix run.
The original Function move is an exact 389-line body match after normalizing
only the five public enum arm headers. The later hidden-invoker move preserves
its complete body after normalizing only that sixth arm header. Compile,
focused constructor/call/apply/bind/toString and bound-call/construct fixtures,
and the real `built-ins/Function` shard remain queued behind the same matrix
run.
The URI move is statically source-equivalent after normalizing only the closed
enum path and rustfmt's block-expression layout, and is boundary-checked; its
compile, focused global-codec fixtures and real URI/Annex-B shard gates remain
queued behind the same matrix run.
The global numeric move is statically source-equivalent after normalizing only
the closed enum path, and is boundary-checked; its compile, focused coercion
and cross-realm fixture gates and real `isFinite`/`isNaN` shards remain queued
behind that matrix run.
The Error move preserves the existing emitter and local-allocation sequences;
its only semantic-free rewrites replace raw builtin-ID tests with the closed
`ErrorBuiltin` and `NativeErrorKind` domains. Its compile, focused constructor,
cross-realm, static predicate and prototype-method fixtures, and real Error,
NativeErrors, AggregateError and SuppressedError shards remain queued behind
the same matrix run.
The JSON move is a verbatim body extraction after normalizing only the closed
enum path and rustfmt layout. Its compile, focused parse/reviver, stringify,
raw-JSON and cross-realm gates, and real `built-ins/JSON` shard remain queued
behind the same matrix run.
The Number move is statically source/token-equivalent for the four predicates,
both split constructor paths, shared receiver path, six prototype operations and
temporary-local releases. Its static boundary/task/diff/rustfmt gates are green;
compile, focused fixture, pre/post golden and real `built-ins/Number` gates remain
queued behind the same matrix run.

### Landed 2026-07-31: the `intrinsics/` boundary

`crates/lila-aot-wasm/src/intrinsics/` now holds per-family realm bootstrap
and property-descriptor installation, extracted from
`builtins/bootstrap.rs::init_builtin_constructor_object`. That function was a
single ~4,760-line body and the worst merge point in the backend: two lanes
adding builtins to unrelated families still collided inside it.

`bootstrap.rs` went from 8,080 to 4,117 lines; 23 dispatch arms became one-line
delegations into 15 family modules. The boundary is enforced by
`check-module-boundaries.sh`.

Every arm moved **verbatim** — installers destructure an `IntrinsicInstall`
context back into the original identifier names (including `builtin`, which
multi-variant arms branch on), so no body text was rewritten. The move was
verified byte-identical across all 527 CLI fixtures with
`crates/lila-aot-wasm/tests/emit_golden.rs`, which matters because property
installation order is observable through `Object.keys` and the ordinary suites
assert on program output rather than emitted bytes.

The earlier intrinsic split left three immediate follow-ups. All now have
bounded owners:

- **Resolved 2026-08-12:** the append-only no-op dispatch is gone. The standard
  builtin catalog requires a closed installer class on every row, and
  `bootstrap.rs` consumes it through an exhaustive installer match.
- **Resolved 2026-08-12:** the parallel `StandardBuiltinId` tables are one
  catalog with compile-time ordering and uniqueness invariants.
- **Resolved for Object, Proxy, Math, Symbol, BigInt, Boolean, Number, Function,
  global numeric, URI, Error and JSON 2026-08-13:** their bodies are family
  modules; Reflect already has the same boundary. Other large inline families
  should follow the same exhaustive-delegate shape.

### Landed 2026-08-12: catalog-owned bootstrap routing

`init_builtin_constructor_object` performs common function/prototype setup for
every initialized `StandardBuiltinId`, but only 34 IDs then run one of 33
family intrinsic installers. Before this seam that distinction was encoded
backwards: the 33 productive arms were followed by no-op arms naming every
other ID. Adding a builtin compiled only after someone appended its name to an
unrelated no-op tail, while the catalog that owned the builtin could not say
whether an installer was required.

The landed seam is a mandatory catalog field whose value is a closed
`StandardBuiltinInstaller` domain. `None` must skip family dispatch; every
other case must be consumed by an exhaustive backend match that invokes the
corresponding installer. This is behavioral routing rather than passive
metadata: omitting the field makes a new catalog row fail to parse, and adding
an installer variant makes the backend fail to compile until it handles the
case. The existing catalog/function iteration and the location of dispatch
after common setup remain unchanged, preserving construction and observable
property-installation order.

The catalog now records 35 productive roots across 34 installer classes and
745 explicit `None` choices. `ArrayBuffer` and `SharedArrayBuffer` deliberately
share one class because their installer branches on the carried builtin ID.
The backend match contains only the productive classes; the former raw-ID
no-op groups were deleted, reducing `builtins/bootstrap.rs` from 4,903 to 4,156
lines. A catalog contract pins the productive root sequence, and the module
boundary audit requires both the mandatory field and the typed backend
dispatch. The focused catalog contract and central feature-enabled CLI compile
are green; broader behavioral suites remain part of this task's acceptance
gate.

## Objective

Split the current monolithic compiler implementation into stable ownership boundaries without changing JavaScript behavior or emitted semantics. At the time this plan was written, `lila-ir/src/lib.rs` and `lila-aot-wasm/src/lib.rs` are tens of thousands of lines and are the primary merge-conflict bottleneck.

## Required module boundaries

The exact filenames may change, but the resulting architecture must expose equivalent boundaries.

### `lila-ir`

- `ir/`: public `ProgramIr`, statements, expressions, functions, classes, properties, shapes, value information and IDs.
- `lowering/`: AST-to-spec-IR lowering, split by declarations, expressions, statements, functions/classes and modules.
- `early_errors/`: checks that are not delegated blindly to parser diagnostics.
- `builtins/`: builtin IDs, metadata, intrinsic ownership and feature registration.
- `analysis/`: scope/capture analysis, static shape/value analysis and unsupported-feature reporting.
- `operations/`: typed representations of shared ECMAScript abstract operations consumed by backends.
- `diagnostics/`: structured diagnostic codes and source locations.

### `lila-aot-wasm`

- `module/`: sections, imports/exports, tables, globals, data and validation.
- `abi/`: tagged values, call/construct convention, completion convention and host imports.
- `heap/`: allocation/layout/string/object/environment storage and memory growth.
- `emit/`: statement/expression/control-flow dispatch.
- `operations/`: code generation for shared abstract operations.
- `objects/`, `functions/`, `environments/`: internal-method emitters.
- `builtins/<family>.rs`: separate builtin families such as array, string, regexp, typed-array, date and intl.
- `intrinsics/`: realm bootstrap and property descriptor installation.

Keep a small `lib.rs` that re-exports the public API and invokes the top-level pipeline.

## Implementation sequence

1. Add characterization tests before moving code: compile representative fixtures, validate Wasm, execute with the project's lower-bound runtime (experimental Wasmtime feature set per `AGENTS.md`), and record observable outputs/completion kinds. Do not gate characterization on engines that lack the lower-bound features (Wasm GC, typed function references, `exnref`).
2. Extract pure data types and constants first, then helpers, then large emit branches.
3. Replace magic global/table/layout numbers with named registries that can assert uniqueness and stable ordering.
4. Use private modules and narrow `pub(crate)` interfaces. Do not make every helper public to avoid import work.
5. Preserve error text only where tests rely on classification; otherwise prefer stable diagnostic codes over brittle strings.
6. Move tests beside the module they cover while retaining end-to-end crate tests.
7. Keep each extraction commit buildable so regressions are bisectable.

## Non-goals

- No new builtin coverage.
- No redesign of the value representation or completion ABI; those belong to T04/T05.
- No bulk formatting of legacy JavaScript code.
- No generated Test262 count changes beyond accidental stale-status cleanup.

## Acceptance criteria

- Both giant `lib.rs` files become orchestration/re-export surfaces rather than implementation stores.
- Feature families have clear files that separate agents can own.
- There are no cyclic module dependencies or duplicate constant registries.
- Public APIs used by `lila-engine` remain coherent and documented.
- Representative emitted artifacts behave identically before and after extraction. If byte identity is not practical, compare imports, exports, validation, output, completion kind and thrown error class.
- Workspace compile time and binary size do not regress materially solely because of module movement.

## Required tests

```sh
cargo fmt --all --check
cargo check --workspace
cargo test -p lila-ir --quiet
cargo test -p lila-aot-wasm --quiet
cargo test -p lila-engine --quiet
cargo test -p lila-cli --quiet
./target/debug/lila test262 run language/wasm/pass \
  --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 \
  --execution-backend wasm
```

Also run several previously green real Test262 filters from different families to detect moved-helper regressions.

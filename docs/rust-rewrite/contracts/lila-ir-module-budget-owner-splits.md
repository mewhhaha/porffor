# Lila IR module-budget owner splits

Status: implemented T02 ownership and effect-admission boundaries.

## Optional chains, conditional facts and object literals

The 2026-10-06 source successor gives twelve consumed optional-chain methods
one private `lowering/optional_chain.rs` owner. Its initial Reference domain
also closes the private optional-call gap described in the
[private Reference contract](optional-private-call-reference.md). The existing
optional-call source-authority control reads that real owner and retains its
exact source-proof construction and borrowed-transport obligations.

`lowering/conditional_flow.rs` owns the captured conditional facts, equal-map
intersection and ten complete capture, installation and join methods. The
captured fields stay private to their lifecycle owner. Ordinary branch,
short-circuit, environment and value-analysis consumers reach its narrow
parent-visible methods directly.

`lowering/object_literal.rs` owns the complete ordered object-literal walk,
method admission, computed-key shape decision, string-key shape insertion and
duplicate prototype-setter validation. Property ordering, accessors, spread,
class name inference and abrupt paths retain their existing algorithms. The
generic `lower_object_property_key` read/effect authority remains shared in
the parent; it is not an object-literal helper.

Both latter extractions preserve complete bodies and literal bytes. Module
declarations attach real private consumers, without forwarding APIs or
implementation reassembly. The original 605 test bodies remain; one new T09
control expands the unit inventory to 606. No mirrored extraction tests are
added. Source comparison and isolated formatting do not establish executable
equivalence; current combined type, focused, artifact-equivalence and broad
acceptance remain pending. Emitted artifact equivalence, measured compilation and broader ownership remain
separate T02 acceptance work.

## Signature evidence and declaration families

The 2026-10-06 source successor extracts 32 complete propagation, exact-context,
callback and signature-evidence methods into private
`lowering/signature_evidence.rs`. Their pass ordering, finite propagation limit,
observation joins, capture authority, target canonicalization and return evidence
remain the original algorithms. Methods used by the parent or another lowering
family are visible only within `lowering`; internal evidence joins remain private.

`lowering/var_declaration.rs` owns the four complete var statement, For-head,
declarator-list and declarator methods. `lowering/function_declaration.rs` owns
ordinary, generator, async and async-generator declaration publication plus the
Annex B copy. `lowering/lexical_declaration.rs` owns the full lexical initialization
walk and its consumed pending InitializeBinding obligations. Suspended
initializers, patterns, borrowed eval variables, Object Environment ordering,
callable registration and binding facts retain their existing implementations.
The parent keeps the exhaustive `lower_declaration` dispatcher, generic parameter
lowering, resource-declaration admission and shared binding/property authorities.

These are whole-body source extractions with four direct private module
attachments. No forwarding facade, duplicate algorithm, public helper or new
behavioral test is introduced. Exact predecessor groups and the moved method
inventory are retained in the source receipt. Isolated formatting and source
comparison do not establish execution or artifact equivalence; joined type,
focused and broad verification and measured T02 acceptance remain pending.

## Public facade and test source ownership

The public IR facade is a 264-line declaration/reexport surface. Production
content ends before its final `#[cfg(test)] mod tests;`; the former 21,544-line
file put 605 unit tests and their private support inside one inline module. Raw
file size therefore overstated the remaining production work in this facade.

`src/tests/mod.rs` registers 36 coherent test families and one shared private
support source. Families cover entry/module roots, environment analysis,
references, operators, destructuring, object/class/capture ownership, generators,
async control, native invocation, global effects, prepared eval, collections,
Annex B, templates and disposal. Its ordered whole-item `include!` declarations
keep the existing `tests::name` namespace and private access without forwarding
functions or public support APIs. They exist only inside the test-gated module;
production roots and feature owners continue to prohibit `include!`/`#[path]`
implementation reassembly.

The extraction retains all 605 names, assertions and embedded source literals.
Shared exhaustive walkers remain test-private in `src/tests/support.rs`. The
module policy must audit their test obligations separately from actual product
lowering and emission consumers. The Annex B source control binds to the real
`src/tests/annex_b.rs` owner instead of looking in the public facade. Existing
exact test filters remain valid; no new test duplicates the extraction.

Source comparison and isolated formatting establish only source authorship.
The complete successor still requires its combined type check, affected controls
and broad verification. This physical extraction supplies no new conformance
counts, runtime result or full T02 completion.

## Callable source representation

`lila-ir/src/builtins/callable_to_string.rs` owns the closed
`CallableToStringRepresentation` domain, its exhaustive `materialize` consumer,
and the focused behavior test. The public variants remain `ExactSource`,
`NativeNamed`, and `NativeAnonymous`; the parent `builtins` module keeps the
canonical public re-export used by the crate facade.

The extraction gives this independently used public type a real owner instead
of moving the surrounding test module merely to satisfy a raw-line budget. The
parent is 1,748 raw lines, below its 1,760-line cap, and the bounded child is 38
lines.

## Invocation-effect proof lifecycle

`lila-ir/src/lowering/invocation_effects.rs` owns
`AccountedInvocationEffects`, `StandardBuiltinCallAnalysis`,
`AnalyzedInvocationEffects` and `InvocationCallerFlowEffects`. The module is
private, its consumers import the sibling-visible types directly, and the
former `builtin_call_info` owner does not re-export a compatibility path.

`AccountedInvocationEffects` is non-`Clone`, non-`Copy`, and `#[must_use]`.
`recorded()` is the only producer of an unattached proof. Combining proofs
consumes both values, attaching a proof consumes it with emitted call IR, and
the `Drop` implementation rejects a proof that reaches the end of its lifetime
unconsumed. `StandardBuiltinCallAnalysis` carries the proof with the result
that requires it, so callers cannot silently keep the result while discarding
the accounting obligation.

`AnalyzedInvocationEffects` is the closed post-analysis state:
`AlreadyApplied` means lowering has already invalidated the relevant facts,
while `MustAttach` carries the linear proof to emitted call IR. Exhaustive
combination and emission replace the former ambiguous optional carrier, so an
emitter cannot infer whether analysis ran from `None`.

`InvocationCallerFlowEffects` is the opaque, nonduplicable aggregate used by
direct, candidate, construct and forwarded calls. It can be formed from the
opaque source proof, the exhaustive host classification, or conservative
invalidation. `CreateRealm` is the only host builtin admitted as preserving;
`DetachArrayBuffer` remains invalidating even though it does not synchronously
invoke source code.

The standard-builtin catalog is also the shared authority for Object/Reflect
proxy effects. Every modeled operation that can dispatch a trap declares
synchronous user code there, including operations whose exact result branch
already invalidates facts, because spread and mixed-candidate calls do not run
that branch. Exact branches may bypass the fallback only when their current
proof excludes proxy dispatch.

That authority also owns the complete Promise caller-flow partition: 24 of the
29 Promise builtin identities are synchronously effectful and five internal
identities are synchronously pure. `lowering/promise_caller_flow.rs` converts a
call into the closed three-way `PromiseInvocationPolicy`; construction and
resolving-function bypasses are admitted only from its call-context and
primitive-kind proofs. `Function.prototype.apply` is effectful independently
of its forwarded target because converting its array-like argument can
dispatch getters or Proxy traps.

Argument evaluation exposes a separate must-consume `LoweredCallArguments`
authority. It records whether the effect epoch advanced, clears heap shapes on
all earlier arguments, and requires each caller to identify any pre-argument
callee or receiver snapshots before extracting the argument vector. Direct
`this` observations occur only after that consumption. Optional-chain lowering
uses the same boundary while analyzing properties in source order, retaining
the captured callee identity but widening a receiver that later arguments can
mutate.

The former exhaustive builtin result table is 2,248 raw lines, below its
2,250-line cap. The Promise policy owner is 49 lines against a 70-line cap, and
the lifecycle owner is 192 lines against a 210-line cap.

## Source-call caller-flow proof

`lila-ir/src/source_call_flow_proof.rs` owns the only conversion from finalized
`FunctionParamIr` values plus a `BlockIr` to proven caller-flow preservation.
Its private state is carried by `SourceCallFlowEffects`; lowering can observe
or combine that state but cannot construct `ProvenNoFlowInvalidation`. That
state requires a private, non-`Clone`, `#[must_use]`
`ProvenNoCallerFlowInvalidation` token.

The proof visits every parameter default and finalized function body
independently and exhausts all 34 `StatementIr`, 83 `ExprIr` and 29
`SpecOperationIr` variants without a catch-all. Calls, writes, property hooks,
object-capable coercion, iteration, spread/destructuring, suspension, disposal
and class execution reject the proof. Primitive-only coercions are admitted
from `KindSet::PRIMITIVE_ONLY`; deferred function values do not make their
enclosing body effectful because their bodies receive separate proofs.

Source signatures begin unobserved, so calls made before a body proof remain
conservative. Candidate joins can preserve a proof only when every callable
candidate carries one; open targets, missing signatures and indexed-receiver
mutators invalidate caller facts. The standard-builtin catalog owns the exact
13 indexed-receiver mutators, and a colocated contract test pins that set.

Optional-chain property analysis is similarly closed: a proven ordinary data
read preserves facts, while accessors, dynamic keys and unknown shapes may run
user code. This prevents an effect-free primitive prototype read from erasing
intrinsic identity during specialization without weakening getter handling.
Class body observations are reset before the current class elements execute
and merged monotonically afterward. Base constructor effects include every
present instance field and auto-accessor initializer; synthetic derived
constructors are invalidating because their emitted body performs an implicit,
dynamically resolved `super` construction.

The proof owner is 769 lines against an 800-line cap. Class-definition
lowering is 1,458 lines against its 1,500-line cap.

## Static String binding-fact identity

`lila-ir/src/lowering/static_string_binding_facts.rs` owns the flow map from
binding storage identity to proven String value. Its raw `BTreeMap` is private
to the child; the parent and sibling lowerers can read, write or invalidate a
fact only by supplying `BindingInfo`. Two same-spelled bindings therefore
occupy different entries, and popping an inner lexical scope removes only its
entry instead of exposing its value through the outer binding.

The owner has five operations: binding-owned `get`, `insert` and `remove`,
whole-flow `clear`, and equal branch `intersection`. It is 33 raw lines against
a 45-line cap. The strengthened post-scope JSON regression requires the outer
`[1]` fact to remain available after an inner `[2]` shadow exits; a conservative
dynamic fallback is no longer treated as the architectural target.

## Durable enforcement

`invocation_effects_owner_structure` pins the private module, all three closed
owners, canonical proof constructor, single raw unattached state,
nonduplicability, `Drop` boundary, absence of the optional carrier and absence
of a compatibility re-export. `check-module-boundaries.sh` independently
performs tree-wide sole-owner censuses, checks the narrow callable
representation re-export and colocated behavior test, rejects
`include!`/`#[path]` disguises, keeps the storage-key map child-private, requires
the `BindingInfo` point-operation signatures, verifies the complete source-flow
IR census and nonduplicable proof token, pins the indexed-mutator catalog test,
keeps raw predecessor snapshots inside the must-consume argument authority,
pins the Promise invocation-policy owner, and caps the focused children as well
as their parents.

The callable representation extraction is source-equivalent. The effect-owner
follow-ups change compiler analysis, but do not claim a conformance-count
change.

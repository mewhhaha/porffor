# Lila IR module-budget owner splits

Status: implemented T02 ownership and effect-admission boundaries.

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

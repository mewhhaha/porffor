# Dynamic-source AOT capability boundary

## Decision

Lila's Wasm-AOT artifact never contains a parser, interpreter, or VM. Source
that is not known until execution therefore remains a compiler capability gap,
not an ECMAScript rejection and not a passing conformance result. Source proven
at AOT time may eventually be compiled only through the ordinary
front-end-to-Wasm pipeline.

`%eval%` does not always evaluate source. `PerformEval` returns its argument
unchanged before selecting a realm or environment when the argument is not a
primitive String, and `%eval%` called with no arguments returns `undefined`.
Those branches are ordinary builtin execution, not dynamic-source support, and
are admitted by the closed proof below.

This contract does not implement that static subset. It makes the current gap
compiler-visible without inferring support from test paths or source snippets.

## Closed domain

`DynamicSourceKind` names the semantic operation:

- direct `eval`;
- indirect `eval`;
- realm `evalScript`;
- one of the four Function-family constructors.

`DynamicSourceGap` has private fields. Its constructors derive the requirement
from the operation:

- source not proven at AOT time requires runtime compilation;
- AOT-known direct eval requires a caller-environment lowering seam;
- AOT-known indirect eval, realm evaluation, and Function-family construction
  require a target-realm environment lowering seam.

Call sites cannot construct a mismatched pair such as Function construction
plus a caller eval environment. `UnsupportedFeature::DynamicSource` is carried
by `IrDiagnostic`; consumers classify the typed value and do not parse its
display text.

`DynamicSourceIntrinsic` is the closed identity catalog for dynamic-source
operations that are callable compiler intrinsics but are not executable AOT
builtins. It maps the four Function-family constructors and realm
`evalScript` to stable function IDs. The ordinary Function row reuses
`StandardBuiltinId::FunctionConstructor`; the three derived rows and realm
evaluation are compiler-only identities. They acquire no Wasm emitter: every
known call or construction candidate first produces
`UnsupportedFeature::DynamicSource`, and a program with that diagnostic is
rejected before backend planning. The bounded `Function.prototype.call`
forwarding slice described below carries the underlying identity. Other
forwarding callables remain explicit accounting debt.

## Product-path invariants

1. A diagnostic is emitted after lowering resolves a compiler-owned intrinsic
   identity, or when unknown user code erased the global `%eval%` value fact
   without proving that it replaced or deleted the intrinsic. Identifier
   spelling alone is not proof.
2. Direct eval is distinguished from indirect eval only when the original AST
   callee is the `eval` identifier and the lowered callee remains a direct
   global reference. A definitely replaced target, alias, property call, comma
   call, or optional call cannot acquire caller-environment authority.
3. A source proof is constructed from syntax before lowering. It accepts
   primitive string literals, no-substitution templates, parentheses and pure
   concatenations of those forms. Static string facts obtained by folding any
   other expression are not proofs because its evaluation or coercions may be
   observable.
4. Function-family arguments are AOT-known only when every argument has that
   syntax proof and there is no spread. Parameter and body strings will still
   require separate parser goals before any subset can be enabled.
5. The typed diagnostic is a compiler gap. It has no early-error code or native
   error type and cannot satisfy a negative Test262 expectation.
6. The existing zero-argument Function-constructor shortcut is not static
   compilation: it manufactures the wrong callable. It is rejected through the
   same typed boundary until the real target-realm path exists.
7. Generator, async and async-generator function object shapes carry their
   respective constructor identity through the intrinsic prototype's
   `constructor` property. The identity follows aliases and property reads; a
   source identifier named `GeneratorFunction` is not evidence by itself.
8. Optional calls retain the same pre-lowering source proof as ordinary calls.
   Reanalysis of an already-lowered optional-chain prefix is marked as already
   accounted, so it cannot silently downgrade or duplicate the diagnostic.
9. The Test262 harness obtains realm `evalScript` from one typed host builtin
   admitted by `HostSurfacePolicy::Test262`. Product lowering cannot resolve
   that global, and the harness stores the resolved function value directly on
   `$262`, preserving literal-source proof at its eventual call site. The host
   body is a defensive throw only; a resolved call is rejected by the compiler
   diagnostic before backend planning.

The private `DynamicSourceProof` is a non-`Clone`, non-`Copy` two-row authority
with seven lexical type mentions. Syntax classification produces it once and
the sole exhaustive gap projection consumes it into either runtime-compilation
or AOT-known environment debt. It has no debug, equality, cast, wildcard or
default route, so downstream lowering cannot duplicate, compare or reuse one
source proof after diagnostic ownership has been transferred. This is a
source-equivalent capability closure: it changes no accepted syntax,
diagnostic, lowered IR or dynamic-source support claim.

`dynamic_source_proof_structure.rs` pins the exact declaration, complete
producer, final construction route and sole exhaustive gap projection. Its
focused structure target passes `3/3`; the existing closed-operation and
requirement owner witness passes `1/1`.

The private, non-`Clone`, non-`Copy` `UnsupportedDynamicSourceCall` owns the
resolved builtin-accounting identity and `DynamicSourceGap` as one one-shot
authority. Resolution derives both fields together; consumers can move the
authority or discard an already-accounted result, but cannot construct or
separate the pair. The sole recorder decomposes it, applies the already-derived
standard-builtin accounting projection and emits the typed diagnostic.
Construct lowering no longer carries a redundant function ID beside that
authority. This is a source-equivalent
unsupported-accounting closure: it changes no diagnostic, builtin count,
lowered IR or supported dynamic-source subset.

`unsupported_dynamic_source_call_structure.rs` pins the exact private pair,
sole construction route, sole decomposing recorder, six diagnostic-recording
call sites and the distinct already-accounted discard route.
Focused evidence is:

```sh
cargo test -p lila-ir --test unsupported_dynamic_source_call_structure -- --test-threads=1
cargo test -p lila-ir dynamic_source_diagnostics_carry_closed_operation_and_requirement -- --test-threads=1
```

Direct-eval context now carries a private, non-`Clone`, non-`Copy`
`DirectEvalCallSite`. Its sole producer requires both the intrinsic `%eval%`
identity and direct global-reference syntax before constructing the witness.
The erased-target classifier additionally requires the original `eval`
identifier, an erased function-capable value and global-property provenance
that still permits the intrinsic; it reuses the same private witness and source
proof. An open target set is captured even when it retains other known
candidates; a retained `%eval%` candidate is instead handled by the shared
candidate phase, so the residual and known routes cannot duplicate the gap.
Sibling lowering modules can still select ordinary call, construct and RegExp
literal contexts, but they cannot manufacture caller-environment
classification. One exhaustive borrowed projection maps the witnessed row to
direct eval and every other call-site row to indirect eval. This closes the
former Wasm-ready paths where an earlier user call erased `%eval%` completely
or left only a different known candidate before a String-source invocation.

`direct_eval_call_site_structure.rs` pins the exact non-capability domain, sole
producer, one-shot erased identity and exhaustive projection. Focused evidence
is:

```sh
cargo test -p lila-ir --test direct_eval_call_site_structure -- --test-threads=1
cargo test -p lila-ir dynamic_source_diagnostics_carry_closed_operation_and_requirement -- --test-threads=1
cargo test -p lila-ir an_eval_property_call_has_indirect_eval_authority -- --test-threads=1
cargo test -p lila-ir direct_eval_after_a_user_constructor_keeps_its_typed_dynamic_source_gap -- --test-threads=1
cargo test -p lila-ir direct_eval_callee_is_captured_before_an_argument_replaces_the_global -- --test-threads=1
```

The structure target passes `7/7`. The erased identity is captured before
argument lowering and consumed afterward, so an argument-side replacement
cannot retroactively change which function Reference the call already
obtained. This is an evaluation-order-preserving direct-eval closure.

## Proven no-source `%eval%`

The lowering boundary classifies each resolved dynamic-source call exactly once
as either `EvalPassThrough(ProvenEvalPassThrough)` or
`Unsupported(UnsupportedDynamicSourceCall)`. The pass-through proof has private
constructors and exists only for direct or indirect intrinsic `%eval%` when:

- the call has no spread and no arguments; or
- the call has no spread and its lowered first argument has a nonempty
  `KindSet` that excludes primitive `String`.

An empty kind set is not evidence. A set containing `String`, any spread,
realm `evalScript`, and every Function-family identity remain typed gaps. An
exact multi-target call requires every dynamic-source target to produce the
pass-through proof.

Target completeness is independent of `heap_shape` and lives in the closed
`FunctionTargetKnowledge::{Exact, Open}` lattice. `Exact(targets)` states that
the set is exhaustive; `Open(targets)` retains known candidates while admitting
additional targets. Joining two values unions their candidate sets and remains
exact only when both inputs are exact. A possible replacement widens either
variant to `Open` without discarding its known candidates. Heap-shape joins may
therefore erase incompatible shapes while exact target joins remain exhaustive.
Lowering may inspect `known_targets()` for conservative effect or
dynamic-source accounting, but exhaustive multi-target dispatch requires
`exact_targets()` and backend single-target specialization requires
`exact_single_target()`. An open set never grants either authority.

One shared candidate-analysis phase consumes this lattice for ordinary,
property, private, super and optional calls and for construction. It visits all
retained candidates through `known_targets()`, preflights dynamic-source
identities before any candidate effect can erase facts needed by a later
alternative, and adds a generic residual result and unaccounted effects only
for `Open` knowledge. For `Exact` mixed values, non-callable or
non-constructable branches throw and therefore add no normal result. A known
candidate whose signature is not registered yet can occur while class static
elements execute; that branch is explicitly treated as unaccounted instead of
silently retaining precise facts.

Construction uses the same phase but keeps the ECMAScript normal-result
codomain object-like: primitive explicit returns are projected away before
joining with the constructed receiver, reusable exact-context return facts are
consumed, literal Proxy trap hints are installed before argument lowering, and
an evaluated callee's proven common `prototype` refreshes each source
constructor instance. When no common current prototype is proven, the
definition-time prototype is removed while retaining known own instance
properties. Spread arguments widen source parameters and source-call results;
spread construction stays within a generic object-like result instead of
reusing an earlier narrow return observation. A standard builtin without a
specialized result summary falls back to its declared signature rather than
silently removing that candidate's normal result.

Consequently, exact aliases with differing function shapes preserve their full
target set through control-flow joins. Multi-target `%eval%` classification
applies the pass-through rule to every target and merges all result facts, so a
no-argument target contributes a genuine `undefined` alternative rather than
erasing the other target's result.

The proof permits lowering to retain the ordinary indirect call. It never
replaces the call with its first argument or `undefined`: the evaluated callee
and every argument remain in source order, an overwritten `eval` still wins,
and abrupt argument completion is unchanged. Its result fact is `undefined`
for no arguments and the first argument's exact `ValueInfo` otherwise.

This is intentionally not an AOT-known textual subset. No String source is
parsed, compiled, or executed by this branch, and it establishes none of the
caller-environment, target-realm, declaration-instantiation, or deferred-error
capabilities required by static Script evaluation.

## Function.prototype.call forwarding authority

The spread-free intrinsic `Function.prototype.call` forwarding path now sends
the evaluated receiver's retained function targets, the original arguments
after `thisArg`, and their lowered values through the same dynamic-source
candidate preflight as an ordinary call. Its call-site context is always
indirect: owning `%eval%` as the forwarded receiver cannot manufacture the
private direct-reference witness. Literal source therefore produces the
AOT-known indirect-eval or Function-family target-realm gap, runtime-derived
source produces the runtime-compilation gap, and missing or proven non-String
`%eval%` input retains its exact pass-through result.

The route is considered only while `Function.prototype.call` acquisition
remains proven intrinsic. Direct mutation of that property, replacement of the
receiver's prototype, or unknown user-code effects erase that authority before
candidate preflight. An `Open` receiver without a retained heap shape therefore
does not reach forwarding preflight merely because one known target is `%eval%`.

That shared boundary returns the closed, must-use `DynamicSourceCallAdmission`.
`Rejected` returns before forwarded `this`, parameter or caller-flow
observations and before call IR emission. `Admitted` owns the exact retained
candidate list and `%eval%` pass-through results; private fields prevent a
sibling lowering path from manufacturing admission. Both `Exact` and `Open`
targets are preflighted through `known_targets()`, while only an exact single
target may contribute the narrow pass-through result and suppress the
underlying eval caller-flow effect.

This slice deliberately requires a proven absence of spread because source
syntax and lowered forwarded positions must stay one-to-one. `apply`,
`Reflect.apply`, `Reflect.construct`, bound functions and proxies remain
explicit forwarding debt. Their closure requires typed argument-list or
underlying-target authority; neither identifier spelling nor an open prototype
shape is sufficient.

Focused ownership and behavior targets are:

```sh
cargo test -p lila-ir --test forwarded_dynamic_source_call_structure -- --test-threads=1
cargo test -p lila-ir --test forwarded_dynamic_source_call -- --test-threads=1
```

## Current producer coverage

| Operation | Compiler-owned identity today | Accounting |
| --- | --- | --- |
| direct/indirect `%eval%` among known call candidates, spread-free intrinsic `Function.prototype.call` forwarding, plus open direct global references where the intrinsic remains possible | `StandardBuiltinId::EvalFunction` | no-argument/proven non-String pass-through; typed diagnostic whenever String remains possible |
| ordinary `%Function%` among known call/construct candidates and spread-free intrinsic `Function.prototype.call` forwarding | `StandardBuiltinId::FunctionConstructor` | typed diagnostic |
| Generator/Async/AsyncGenerator Function constructors among known call/construct candidates and spread-free intrinsic `Function.prototype.call` forwarding | `DynamicSourceIntrinsic::Function(..)` carried by the function prototype shape | typed diagnostic |
| known `$262.evalScript` call candidates | `HostBuiltinId::RealmEvalScript`, mapped to `DynamicSourceIntrinsic::RealmEvalScript` and exposed only by `HostSurfacePolicy::Test262` | typed diagnostic |

There is no lexical Test262 pre-gate for these operations. Unsupported
accounting begins only after lowering resolves one of the identities above.
This does not claim static-source support: literal strings still produce
caller- or target-realm-environment debt, while values that may be primitive
Strings produce runtime-compilation debt. Only proven no-source `%eval%` calls
avoid the dynamic-source diagnostic.

## Optional-call accounting authority

The private, capability-free `OptionalCallSource` couples source-proof
availability and diagnostic ownership for each optional call. `Syntax` borrows
the original parser arguments and owns the resulting dynamic-source diagnostic;
`AlreadyAccounted` carries no syntax and suppresses a duplicate diagnostic when
an already-lowered optional-chain prefix is analyzed again.

The chain analyzer borrows each authority exactly once and exhaustively maps it
to the shared `CallCandidateSource` domain. Both rows preserve pass-through
result facts. An unsupported `AlreadyAccounted` row reuses the prior undefined
placeholder without recording again, while a syntax-owning row records the
builtin use and typed gap. Invocation-effect tokens from every optional call
are combined and attached to the emitted chain. There is no copyable
authority, Boolean ownership projection, wildcard or default route.

Focused evidence is:

```sh
cargo test -p lila-ir --test optional_call_source_authority_structure -- --test-threads=1
cargo test -p lila-ir no_source_eval_works_through_alias_optional_and_safe_multi_target_calls -- --test-threads=1
cargo test -p lila-ir grouped_optional_dynamic_source_prefix_is_accounted_once -- --test-threads=1
```

The structure target passes `4/4`; each exact owner witness passes `1/1`.
The first proves that a newly parsed optional call retains pass-through
accounting without a dynamic-source diagnostic, and the second proves that
grouped-prefix reanalysis does not duplicate that diagnostic. Package
formatting and the focused diff check are clean. Independent review hardened
the guard's producer routes, authority cardinality and complete four-row
decision, then finished clean. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module
boundary check and the task-plan check; the compile retains the repository's
existing warnings.

Beyond the bounded `Function.prototype.call` slice, forwarding builtins and
wrapper callables are not yet identity-transparent for this accounting
boundary. Calls that reach dynamic source through `apply`, `Reflect.apply`,
`Reflect.construct`, bound functions or proxies can still fall through to a
generic backend diagnostic. Closing those gaps requires typed forwarding
targets, not source-spelling recognition.

## Static-subset prerequisites

`precompiled-realm-scripts.md` defines the implementation contract for
AOT-proven Script source. In particular, static realm evaluation means a
separate precompiled Script thunk, deferred ECMAScript parse/early errors and
runtime GlobalDeclarationInstantiation. It never means splicing statements
into the caller or manufacturing a declaration-free host result.

Direct eval additionally needs an eval parse goal, delayed parse/early errors,
runtime `%eval%` identity checking, caller variable and lexical environments,
strictness, `this`, `new.target`, private environment, declaration
instantiation, completion values, and realm-correct errors.

Function-family construction needs separate parameter/body parse goals and
combined early errors, ordinary argument evaluation and coercion order, runtime
constructor identity and `newTarget`, and function allocation against the
constructor realm's global environment. Until those seams exist, a literal
source is classified as AOT-known debt rather than compiled by a shortcut.

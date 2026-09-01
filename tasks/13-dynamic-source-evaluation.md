# T13 — Dynamic source evaluation: `eval`, `Function` and realm evaluation

**Status:** Policy, typed accounting, no-source `%eval%` and bounded `.call` forwarding implemented; textual static subsets remain

**Parallel group:** Feature lane; architecture decision recorded
**Depends on:** T03, T06, T08, T09, T12  
**Blocks:** Honest accounting for dynamic-code Test262 cases and parts of T24/T26

## Current repository state

The active product policy is explicit: generic `eval`, Function-family
construction and realm `evalScript` remain visible Wasm-AOT unsupported cases
when support would require an interpreter or runtime parser. Resolved ordinary
`eval` calls that may receive primitive String source and `%Function%` calls
now carry a closed `UnsupportedFeature` through
IR diagnostics into conformance accounting. The three derived Function-family
constructors now have closed compiler-only intrinsic identities carried by
function prototype shapes, and `$262.evalScript` is a typed host
identity admitted solely by the Test262 host-surface policy. Test262 no longer
infers any dynamic-source result from source spelling. The README reports all
of these cases separately.
The spec's pre-source `%eval%` branches are implemented: a no-argument call
returns `undefined`, and a call whose lowered first-argument kind is nonempty
and excludes primitive String returns that value unchanged. This is ordinary
builtin execution, not a textual static subset. Supported statically known
source subsets have not been implemented. Keep this task focused on capability
reporting and general compilation paths rather than treating the permitted
unsupported result as a pass.

The spread-free intrinsic `Function.prototype.call` forwarding path now
preserves the receiver's dynamic-source identity and shifts both original and
lowered arguments past `thisArg` into the shared candidate preflight. This is a
bounded identity/accounting closure, not support for evaluating source.

The original `built-ins/Boolean/S9.2_A1_T1.js` now reaches this boundary
without a Test262 source rewrite. Its sloppy and strict variants both report
the existing `NotImplemented/Runtime` result at `eval("var x")`. They are not
green, skipped or expected failures; they remain literal dynamic-source debt.

Five former T18 String materializers now expose the same compiler-owned
boundary from exact vendored sources. Four direct-`eval` sources—the legacy
`charAt`, `charCodeAt`, `indexOf` and `match` cases—produce the typed
caller-environment gap; the legacy `slice` source uses the ordinary `Function`
constructor and produces the typed target-Realm-environment gap. Across sloppy
and strict modes, the spec-exec oracle passes `10/10`, while Wasm-AOT passes
`0/10` and records all ten as typed `Unsupported`. This is honest capability
accounting, not a conformance regression or a supported static subset;
six adjacent non-dynamic product controls pass all `12/12` sloppy/strict
Wasm-AOT executions.

`built-ins/Proxy/revocable/tco-fn-realm.js` now reaches the same boundary from
its unchanged pinned source. Its raw `other.evalScript` call is no longer
replaced by a Proxy-specific materialization. The synthetic created-realm
record shape types `evalScript` as `HostBuiltinId::RealmEvalScript`, and current
AOT bootstrap work installs a realm-local, self-backed function with that
realm's Function and TypeError prototype identities. Lowering still maps an
invocation to `DynamicSourceIntrinsic::RealmEvalScript` and the closed
`RealmEvalScript` AOT unsupported result before backend planning. This identity
invariant makes the unsupported owner reachable without pretending to execute
the supplied source.

Four former Proxy apply/construct materializers now expose the same honest
boundary from unchanged pinned sources. The two `arguments-realm.js` leaves
retain indirect eval, while the two new-target-Realm construct leaves retain
ordinary Function construction. Their complete host and local assertion/sta
preludes are retained. One exact four-path harness boundary reports all four as
explicit Wasm-AOT dynamic-source gaps because the compiler does not yet type
these created-Realm property calls. These are unsupported cases, not Proxy
passes, and deleting their rewrite owners removes T11 from the shortcut
inventory.

The 2026-08-13 current-pin Wasm-AOT run supplies the first concrete static
subset owner: its first 17 failures are all typed `$262.evalScript`
target-realm-environment gaps, with no timeout or crash. These are not a
declaration-free literal cluster. Sixteen exercise descriptor-sensitive
global/Annex-B declaration instantiation, and the remaining lexical-collision
case requires a deferred `SyntaxError` with no partial `var` mutation.
`docs/rust-rewrite/contracts/precompiled-realm-scripts.md` therefore fixes the
implementation boundary before code changes: one syntax-proven precompiled
Script registry, deferred parse/early-error results, runtime
GlobalDeclarationInstantiation and must-use realm-context restoration. Source
splicing and a declaration-free harness shortcut are explicitly excluded.

## Objective

Resolve dynamic JavaScript source evaluation without violating the project ban on shipping an interpreter/VM inside emitted Wasm. Implement every compliant subset that can remain direct compilation, and report the rest explicitly unless a later architecture decision approves a host-compiler design.

Dynamic `import()` is explicitly not in this task's unsupported bucket: T12's componentized-AOT strategy handles it by resolving specifiers to precompiled module components at runtime. This task covers only textual dynamic source — `eval`, the `Function`-family constructors and realm `evalScript`.

## Architecture decision

**Decision:** Wasm-AOT artifacts do not compile source at runtime. Generic
direct or indirect `eval`, Function-family construction and realm `evalScript`
are explicit unsupported dynamic-code-generation cases. This is a product
capability boundary, not a passing Test262 result.

Source proven constant during AOT compilation may be supported only by sending
it through the ordinary parser, early-error, spec-IR, lowering and Wasm-codegen
pipeline. Such a specialization must preserve direct-eval scope, strictness,
realm ownership and observable argument evaluation; recognizing a test path,
source fragment or assertion is forbidden. This path remains implementation
work and its absence remains visible debt.

An optional Rust host compiler service was considered and is not part of the
1.0 Wasm-AOT contract. It would make otherwise standalone artifacts depend on
an embedding capability and would require a re-entrant bridge for lexical
environments, realms and observable heap identity. It would also make security
policy, caching and deterministic-build behavior host-dependent. Those costs
are not justified while generic dynamic compilation is an explicitly permitted
capability gap. Introducing such a service later requires a new architecture
decision and an explicit typed capability; it may not appear as a silent
fallback.

The alternatives are therefore resolved as follows:

1. **AOT-known source:** permitted as the sole direct-compilation subset, but
   not yet implemented.
2. **Rust host compiler service:** deferred outside the current product
   contract, with no implicit import or fallback.
3. **Generic runtime source:** explicitly unsupported and separately accounted
   for by Wasm-AOT. The spec-exec oracle may execute it during differential
   triage, but that result is never product support or conformance evidence.

Compiling a parser, interpreter or VM into the artifact remains forbidden.
Because the selected path performs no runtime compilation, it preserves
standalone deterministic artifacts, leaves CSP-like policy at a clear
capability boundary and introduces no compiler re-entrancy or cross-instance
heap bridge.

## Typed capability boundary

`docs/rust-rewrite/contracts/dynamic-source-capability.md` is the source of
truth for the closed operation and requirement domains. `DynamicSourceGap` has
private fields: its constructors derive runtime compilation, caller-environment
or target-realm-environment debt from `DynamicSourceKind`. An unsupported
diagnostic carries `UnsupportedFeature::DynamicSource`; consumers match that
enum rather than its display string.

Current compiler producers cover known call/construct candidates for
direct/indirect `%eval%`, all four Function-family constructors and realm
`evalScript`, including spread calls, optional calls across those identities and
zero-argument Function construction. Direct `eval` also retains its typed gap
when an earlier user-code effect erases the global builtin value fact without
proving a replacement; the classifier requires both original identifier syntax
and a lowered direct global reference. The old zero-argument shortcut did not
compile an empty function; it manufactured a value with Function-constructor
metadata, so it is now typed unsupported with every other Function-constructor
call.

Resolved `%eval%` is classified through one private, must-use disposition.
No-argument calls and no-spread calls whose first argument has a nonempty
`KindSet` excluding primitive String receive a `ProvenEvalPassThrough` and keep
their ordinary indirect-call IR. A String-capable or unknown argument, every
spread, realm `evalScript`, and all Function-family operations retain their
typed gaps. The pass-through never folds away the call: runtime callee identity
and evaluation of every argument remain observable. Function-target
completeness is carried independently of heap shape by the closed
`FunctionTargetKnowledge::{Exact, Open}` lattice. Joins union the known target
IDs and remain `Exact` only when both inputs are exact, so differing-shape
function aliases may lose heap-shape facts without losing exhaustive target
authority. A possible replacement widens the knowledge to `Open` while
retaining every known candidate. Only `Exact` knowledge permits exhaustive
multi-target dispatch or a single-target specialization; `Open` candidates
remain available for conservative effect and dynamic-source accounting but
cannot authorize either operation. Exact multi-target `%eval%` calls
therefore apply the pass-through rule to every dynamic-source target and merge
their result facts, including a genuine `undefined` alternative.

A shared candidate-analysis phase now owns ordinary, property, private, super
and optional calls plus construction. It preflights every retained
dynamic-source identity before candidate effects run, analyzes known candidates
for both `Exact` and `Open` knowledge, and reserves generic residual behavior
for `Open`. Non-callable or non-constructable branches of an exact mixed value
throw and do not pollute its normal result. Class static initialization can
publish a candidate before registering its signature; that candidate forces
unaccounted effects rather than preserving unsound facts. The construct path
also projects primitive explicit returns out of the normal result, consumes
reusable exact-context observations, seeds literal Proxy traps before argument
lowering and applies any proven common evaluated-callee prototype. Without a
common current prototype it removes the definition-time prototype fact while
retaining known own instance properties. Spread calls widen source parameters
and results, spread construction remains generically object-like, and a
standard builtin lacking a specialized call summary contributes its declared
signature result instead of disappearing from the join.

The spread-free intrinsic `Function.prototype.call` forwarding route uses the
closed, must-use `DynamicSourceCallAdmission` produced by ordinary candidate
preflight. Rejection occurs before forwarded `this`, parameter or caller-flow
observations and before call emission. Admitted no-source `%eval%` retains its
precise result; all retained `Exact` or `Open` receiver candidates still take
part in dynamic-source accounting, and only an exact single eval target grants
that pass-through precision. The route is considered only while
`Function.prototype.call` acquisition remains proven intrinsic. Direct
mutation of that property, replacement of the receiver's prototype, or unknown
user-code effects erase that authority before candidate preflight. An `Open`
receiver without a retained heap shape therefore does not gain forwarding
authority merely because one known target is `%eval%`. `apply`, `Reflect.apply`, `Reflect.construct`,
bound functions and proxies remain explicit forwarding debt; identifier
spelling must not be used to paper over them.
`forwarded_dynamic_source_call_structure.rs` pins the admission ownership and
pre-observation rejection order; `forwarded_dynamic_source_call.rs` covers
indirect eval classification, all four Function-family identities, captured
receiver order, `Exact`/`Open` candidates and no-source result precision. Their
focused commands are recorded in the capability contract for the coordinated
verification checkpoint.

This boundary deliberately does not claim the static subset. A literal eval
string is recorded as blocked on the caller/realm environment seam rather than
sent through Script parsing, and a Function-family literal is recorded as
blocked on the target-realm environment seam rather than synthesized as a
wrapper. Generic String-capable source remains blocked on runtime compilation;
proven non-String eval is outside that boundary because it evaluates no source.

`DynamicSourceIntrinsic` is the non-executable catalog behind the remaining
identities. Generator, async and async-generator function object shapes expose
the right constructor through their intrinsic prototype; aliases therefore
retain identity without recognizing identifier spelling. The Wasm-AOT Test262
harness stores the Test262-only realm-eval host builtin directly on
`$262.evalScript`, so lowering sees the caller's actual argument expressions.
The compiler-only Function identities have no backend emitter. Realm eval has a
defensive host body so the always-loaded harness can carry a valid function
object, but every directly resolved invocation produces the typed diagnostic
and is rejected before backend planning.

The diagnostic's AOT-known/runtime split is now derived before lowering from a
private closed source-proof boundary. String literals, no-substitution
templates, parentheses and recursively pure literal concatenations are the
only admitted forms. A folded `ExprIr::String` can no longer manufacture the
proof, so observable calls or conditionals that happen to fold to a string
remain `RuntimeCompilation` gaps. This is source ownership for the future
precompiled registry, not execution of the static subset.

That private `DynamicSourceProof` is now a non-`Clone`, non-`Copy` two-row
authority with seven lexical type mentions. Syntax classification produces it
once and the sole exhaustive gap projection consumes it into the typed
runtime-compilation or AOT-known environment debt. The `3/3` structure guard
pins the exact declaration, complete producer, final construction route and
sole exhaustive gap projection; the existing closed-operation and requirement
owner witness passes `1/1`. This is a source-equivalent capability closure:
accepted syntax, diagnostics, lowered IR and the unsupported dynamic-source
surface remain unchanged.

Direct-eval context now carries a private, non-`Clone`, non-`Copy`
`DirectEvalCallSite`. The sole constructor requires both the intrinsic
`%eval%` identity and direct global-reference syntax, so sibling lowering
modules cannot manufacture caller-environment classification for aliases or
property calls. An erased-target classifier admits the same identity only when
the original AST is the `eval` identifier, its value may still be a function,
and global-property provenance has not proved replacement or deletion. The
exhaustive borrowed context projection keeps all other call-site rows indirect.
The `7/7` structure guard pins the domain, sole producer, one-shot erased
identity, and projection. The erased identity is captured before arguments and
consumed afterward, so an argument-side replacement cannot retroactively
change which function Reference the call already obtained. Open knowledge is
captured even when another known candidate remains, unless `%eval%` itself is
already retained for the shared candidate phase. This is an
evaluation-order-preserving direct-eval closure. The exact commands are
recorded in the dynamic-source capability contract.

Optional-chain analysis now carries its pre-lowering source authority through
the private, capability-free `OptionalCallSource` domain. One borrowed
authority couples source-proof availability and diagnostic ownership, then an
exhaustive conversion feeds the shared candidate phase. Already-accounted
prefixes retain their analysis fact without duplicating diagnostics; newly
parsed calls retain their original syntax and own builtin/gap accounting.
Effect tokens from every call in the chain are combined and attached to the
emitted chain. Its bounded structure guard passes `4/4`, and the new-call
pass-through and grouped-prefix no-duplicate-diagnostic owner witnesses each
pass `1/1`. The commands and semantic ownership of both witnesses are recorded
in the dynamic-source capability contract.

Unsupported invocation accounting now moves through the private, non-`Clone`,
non-`Copy` `UnsupportedDynamicSourceCall`. It owns the resolved
builtin-accounting identity and `DynamicSourceGap` as one authority. The sole
constructor derives both together. Consumers may move it or discard an
already-accounted result, but the sole recorder decomposes the pair before
emitting the typed diagnostic. Construct lowering no longer carries a
redundant function ID beside the resolved unsupported result. The focused
structure guard pins that single producer, sole projection route and all six
diagnostic-recording call sites plus the already-accounted discard route. This
is a source-equivalent unsupported-accounting closure: it
changes no diagnostic, builtin count, lowered IR or capability claim. The
focused commands are recorded in the dynamic-source capability contract.

## Semantic scope

### Direct eval

- Determine direct vs indirect call syntactically/semantically.
- Preserve caller strictness, variable/lexical environment selection, `this`, `new.target` and private environment.
- Handle declarations, conflicts and completion values.
- Static-string specialization must use the normal parser/lowering/codegen pipeline and must not recognize Test262 assertion text.

### Indirect eval and realm `evalScript`

- Execute as global code in the target realm.
- Use the target realm's intrinsics and global environment.
- Propagate parse/early/runtime errors with target-realm prototypes.

### Function-family constructors

Cover `Function`, `GeneratorFunction`, `AsyncFunction` and `AsyncGeneratorFunction` constructors, parameter/body parsing, realm selection, names/length/prototypes and syntax errors.

## Requirements for any future host-compiler reconsideration

- Supersede the decision above explicitly rather than adding an incidental call
  from one builtin.
- Use a typed host import rather than a magic `eval` opcode.
- Compile source with the same Rust front/IR/Wasm pipeline.
- Define a state bridge so evaluated code sees and mutates the required environment/realm objects without copying observable identity.
- Cache only when source, realm policy and environment shape make caching unobservable.
- Prevent recursive compilation from corrupting the active Wasm instance.
- Expose a clear error when the embedding host disables dynamic compilation.

## Acceptance criteria

- The repository has one documented policy; no ambiguous fallback.
- Proven no-source `%eval%` retains runtime callee identity and evaluates every
  argument exactly once in source order; unknown, String-capable and spread
  calls remain typed gaps.
- Any supported static direct-eval cases preserve lexical scope and abrupt completions.
- Any supported indirect/cross-realm evaluation never aliases the wrong global.
- Unsupported dynamic cases are classified consistently and remain in real-suite accounting.
- No source regex/materialization exists for known Test262 eval/Function cases.
- If a later architecture decision selects a host service, representative
  dynamic strings—not known at AOT time—pass scope, realm, constructor and
  error tests.
- The README/CLI clearly report artifact capability requirements.

## Required tests

```sh
cargo test -p lila-front eval_ --quiet
cargo test -p lila-ir eval_ --quiet
cargo test -p lila-engine eval_ --quiet
cargo test -p lila-cli eval_ --quiet
```

Run real filters under `built-ins/eval`, `built-ins/Function`, generator/async function constructors, direct/indirect eval language tests and `$262.evalScript` cross-realm cases. Report unsupported counts separately until resolved.

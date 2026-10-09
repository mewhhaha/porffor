# T13 — Dynamic source evaluation: `eval`, `Function` and realm evaluation

## ShadowRealm and retained finite-source parsing — 2026-10-07 dry source

Finite ShadowRealm evaluation now has its own source kind, native entry and
receiver-Realm dispatch, with fresh eval lexical environments and separate
source-syntax versus execution failure handling. A compilation-local lowering
session retains parsed syntax and failures across module-discovery retries;
nested and aliased importValue requests close before artifact cache identity.
The accepted IR is reused for emission. Workspace types, prepared-source cache
controls and the native finite-evaluation lifetime control pass. Optional
forwarding and finite computed-key discovery pass the complete workspace and IR
checkpoint; the optional-alias native cohort passes both modes in 364.18 seconds.
Its queue was deliberately stopped during the second case after 466 seconds
(exit 143), leaving seven cases incomplete or unstarted. Shared Realm
initialization now passes `tasks-realm-bootstrap1` in 210 seconds: source guards,
the all-feature/all-target workspace check (60.36 seconds; Cargo: 59.02) and
all 16 focused AOT controls
(six helper unit, ten integration), with none failed or ignored. The validated
optional artifact's main/initializer/ShadowRealm bodies are 534,595/662,558/24,632
bytes; all bodies stay below 1 MiB and both callers share the helper. Earlier
IR/cache/catalog and native results keep their preceding-source scope.
`tasks-dry-closure-native2` received an unexpected SIGTERM at 188 seconds
(exit 143) during its first strict compile; sloppy execution passed, but no
function completed. Accounting recorded 3.8 GiB peak without an OOM termination
entry. `tasks-dry-closure-native3` passed all six queued ShadowRealm gates,
including finite-source validation, unmatched-source rejection and the nested
module-source catalog. The overall run then received SIGTERM after 5,153 seconds:
30 functions passed, four failed and 19 remained incomplete or unstarted. Service
accounting reported 4 GiB peak without an OOM termination entry. Global-reference
and class-identity repairs are written; their joined checkpoint and the remaining
native queue are pending under the same limits. These results retain their source
revision and do not close broader acceptance.
Arbitrary unavailable source remains a typed runtime capability gap.
See the [finite-source contract](../docs/rust-rewrite/contracts/shadowrealm-finite-evaluate.md).

**Status:** In progress — finite source and shared-initializer type/AOT checks pass; the expanded native run and broader acceptance remain pending, with unmatched runtime source an explicit Wasm-AOT capability gap.

**Parallel group:** Feature lane; architecture decision recorded
**Depends on:** T03, T06, T08, T09, T12  
**Blocks:** Honest accounting for dynamic-code Test262 cases and parts of T24/T26

## Current repository state

The 2026-10-03 dry T13/T06 implementation distinguishes retained parsed units
and sites from fresh finite prepared eval, Realm Script and Function
executions. Lazy template caches belong to those actual executions. Functions,
escaping closures, class elements and suspended functions retain the owner
through their existing immutable contexts. Repeated calls reuse that owner;
separate prepared executions allocate fresh owners. Compilation, regression
sources and the historical tagged-template cache cohort remain pending.
Unmatched dynamic source retains the explicit AOT policy gap. See the
[template ownership contract](../docs/rust-rewrite/contracts/template-site-source-ownership.md).

The September 2026 implementation compiles nonempty prepared sources through
the ordinary parser, early errors, spec IR, lowering and Wasm code generation.
All four Function-family constructors have independently parsed parameter/body
units and real ordinary, generator, async or async-generator bodies. Each call
performs its live argument conversions, matches the complete source tuple and
allocates a fresh function with the active constructor's realm and the required
`newTarget` prototype. See the
[prepared Function contract](../docs/rust-rewrite/contracts/prepared-dynamic-functions.md).

Prepared Script units implement direct eval, indirect eval and realm
`evalScript`. Direct eval retains the original Reference and callable before
argument evaluation, then checks the current realm's immutable original
`%eval%`. Its caller records preserve lexical/variable environment selection,
strictness, TDZ and const behavior, with/unscopables lookup, parameter/body
separation, escaped arrows and permitted `this`, `new.target`, super and private
context. Replaced, foreign, bound or proxied eval calls retain ordinary call
semantics. See [prepared direct eval](../docs/rust-rewrite/contracts/prepared-direct-eval.md)
and [caller Environment Records](../docs/rust-rewrite/contracts/direct-eval-environment-records.md).

The 2026-09-29 for-in replay exposed two strict prepared-eval failures:
`S12.6.4_A3.1.js` and `S12.6.4_A4.1.js` could not resolve the declared loop
variable. Owner analysis now collects `var` for-in bindings before allocating
the strict eval environment, including destructuring names. The added
`aot_direct_eval_environment` regressions retain both exact pinned sources in
both Script modes and check local hoisting, escaped closure cells and isolation
from the caller. Strict direct and indirect eval initialize their planned owned
`var` cells before body execution, including zero-iteration heads; lexical cells
retain TDZ. All 17 tests in that engine target pass, including four executions
of the unchanged pinned sources. The complete for-in replay and broad workspace
verification remain pending.

Indirect eval and realm Scripts use the target realm's global environment and
intrinsics. Runtime declaration instantiation validates conflicts and descriptor
constraints before mutation, creates fresh declared functions and retains
Script completion values. Repeated realm Scripts share their realm binding
state. The [precompiled Script contract](../docs/rust-rewrite/contracts/precompiled-realm-scripts.md)
describes these executable units and their declaration plans.

Candidate discovery includes syntax-proven strings and guarded finite values
from known bindings, literal arrays/records and callback arguments. Function
candidate expansion is bounded at 256 values or source tuples. Discovery does
not replace getters, callbacks, argument effects, coercions or callee lookup;
the actual intrinsic and exact runtime source still decide whether a prepared
unit applies. A malformed prepared unit defers its ECMAScript `SyntaxError`
until invocation. Parser or compiler capability failures remain diagnostics.

The artifact contains no parser, interpreter or runtime compiler. An unmatched
runtime source or an unimplemented source/context combination remains an
explicit typed Wasm-AOT gap. Stateful source generation is not automatically
covered by finite discovery, and a permitted unsupported outcome remains a
failing Test262 execution. No-argument `%eval%` still returns `undefined`,
non-String `%eval%` arguments return unchanged, and zero-argument Function-family
calls still create fresh empty functions.

The final combined repair-cohort replay on `2026-09-09` verified 994 Success,
18 typed unsupported source-discovery outcomes, zero Bug and zero Crash across
1,012 exact executions. The 18 remaining cases are owned by T13 and listed in
the [September repair notes](../docs/rust-rewrite/observed-later-failure-repairs.md).
They cover bounded BMP-generated eval sources and stateful Function parameter
source conversion. This partial cohort does not establish full conformance or
close general dynamic-source discovery.

On 2026-09-28 the `resizableArrayBufferUtils.js` static-subclass
substitution was deleted: the prepared Function-source machinery already
compiles the helper's finite `new Function('return class My' + type + ...)`
candidates, so all 188 vendored consumers now materialize the exact helper
bytes. Exact `lila test262 run` replay of all 188 cases is unchanged versus
the substituted baseline: 185 pass `2/2` sloppy/strict executions and the
same 3 staging cases report typed `Unsupported`. The token-aware inventory
now assigns 0 observations to T13.

## Historical observations before prepared-source implementation

These observations record earlier removal of harness substitutions and the
capability gaps exposed at those checkpoints. Their counts are preserved as
historical evidence; they are not the current outcomes of the prepared-source
implementation.

After removal of its source rewrite, the original
`built-ins/Boolean/S9.2_A1_T1.js` reached `eval("var x")`. Its sloppy and strict
variants then reported `NotImplemented/Runtime`, rather than a substituted
pass or skipped execution.

Five former T18 String materializers exposed compiler-owned gaps from exact
vendored sources. The legacy `charAt`, `charCodeAt`, `indexOf` and `match`
cases reported direct-eval caller-environment debt; the legacy `slice` case
reported ordinary Function target-realm debt. At that checkpoint the spec-exec
oracle passed `10/10` sloppy/strict executions, while Wasm-AOT passed `0/10`
and recorded all ten as typed `Unsupported`. Six adjacent non-dynamic controls
passed `12/12` Wasm-AOT executions. The oracle results did not establish product
support.

The unchanged `built-ins/Proxy/revocable/tco-fn-realm.js` exposed its raw
`other.evalScript` call after removal of a Proxy-specific materialization.
`HostBuiltinId::RealmEvalScript` and realm-local callable metadata made that
operation reachable, but at that checkpoint lowering rejected the invocation
before backend planning. Four other former Proxy apply/construct materializers
likewise exposed four typed gaps: two `arguments-realm.js` leaves used indirect
eval, and two new-target-Realm construct leaves used ordinary Function. Their
host and assertion/sta preludes were retained. Removing those rewrite owners
removed T11 from the shortcut inventory; it did not establish four Proxy passes.

The 2026-08-13 Wasm-AOT run at its then-current pin supplied the first concrete
Script subset: its first 17 failures were typed `$262.evalScript`
target-realm-environment gaps, with no timeout or crash. Sixteen exercised
descriptor-sensitive global/Annex-B declaration instantiation; the remaining
lexical-collision case required a deferred `SyntaxError` without partial `var`
mutation. This evidence motivated the precompiled Script registry, deferred
errors, runtime declaration validation and realm-context restoration now
implemented above. It did not justify source splicing or a declaration-free
harness shortcut.

## Objective

Resolve dynamic JavaScript source evaluation without violating the project ban on shipping an interpreter/VM inside emitted Wasm. Implement every compliant subset that can remain direct compilation, and report the rest explicitly unless a later architecture decision approves a host-compiler design.

Dynamic `import()` is explicitly not in this task's unsupported bucket: T12's componentized-AOT strategy handles it by resolving specifiers to precompiled module components at runtime. This task covers only textual dynamic source — `eval`, the `Function`-family constructors and realm `evalScript`.

## Architecture decision

**Decision:** Wasm-AOT artifacts do not compile source at runtime. Eval,
Function-family construction and realm `evalScript` sources outside their
prepared registry remain explicit unsupported dynamic-code-generation cases.
This is a product capability boundary, not a passing Test262 result.

Source known during AOT compilation, including guarded finite candidates, is
compiled through the ordinary parser, early-error, spec-IR, lowering and
Wasm-codegen pipeline. The implemented prepared units preserve direct-eval
scope, strictness, realm ownership and observable argument evaluation;
recognizing a test path, source fragment or assertion is forbidden. Runtime
dispatch requires the actual intrinsic and matching source. Missing candidates
or unsupported contexts remain visible debt.

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

1. **AOT-known source:** implemented as independently compiled Function and
   Script units with guarded runtime dispatch and the required environment
   records. Candidate discovery and supported contexts remain bounded.
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

`DynamicSourceKind` names direct/indirect eval, realm evaluation and the four
Function-family operations. `DynamicSourceGap` derives a missing runtime
compilation, caller-environment or target-realm requirement from that operation.
`IrDiagnostic` retains `UnsupportedFeature::DynamicSource`; conformance tooling
classifies the typed value instead of parsing its display text. The
[capability contract](../docs/rust-rewrite/contracts/dynamic-source-capability.md)
records these error domains and historical ownership checks; the prepared-unit
contracts linked above describe the implemented execution paths.

Shared candidate analysis consumes the closed `ResolvedDynamicSourceCall`:
`EvalPassThrough`, `IndirectEvalInvocation`, `FunctionInvocation`,
`CompiledScript` or `Unsupported`.
Known Script sources register an executable or deferred-error unit instead of
being rejected solely because they contain text. Resolved Function-family
invocations retain their runtime argument conversions and guarded source
selection even when discovery cannot prepare a matching tuple. In that case,
an abrupt argument conversion remains its real JavaScript exception; successful
conversions followed by an unmatched tuple reach the typed capability gap.

`ProvenEvalPassThrough` admits no-argument calls and non-spread calls whose first
argument has a nonempty `KindSet` excluding primitive String. It retains the
ordinary indirect-call IR, so callee identity and every argument effect remain
observable. Indirect eval with String-capable arguments can instead use
`AdmittedIndirectEvalInvocation`: the live intrinsic returns actual non-String
values unchanged and rejects an unprepared String with a typed runtime
capability failure. A spread cannot obtain the compile-time pass-through proof;
this restriction is not a blanket claim that every spread or forwarding form
fails at runtime.

Required source units and optional candidates have distinct admission. Syntax
proof can require compilation of a unit; finite discovery can register a
candidate without asserting that the runtime callee will use it. Function
parameter/body grammars and Script/Eval grammars are independently parsed, with
caller permissions retained for direct eval. An ECMAScript parse or early error
becomes a deferred realm `SyntaxError`. A parser capability failure or compiler
bug must not be relabeled as that JavaScript error.

The spread-free intrinsic `Function.prototype.call` forwarding route uses the
closed, must-use `DynamicSourceCallAdmission` before observing target effects.
The optimized route is used only while `Function.prototype.call` acquisition
remains proven intrinsic. Candidate discovery also handles supported
literal-array `apply`/`Reflect.apply`/`Reflect.construct` arguments and comma
callees while retaining the original expressions. Finite candidate discovery
preserves runtime callable identity and exact source equality;
it does not imply unrestricted forwarding support. An indirect Script candidate
never grants direct-eval caller context.

`FunctionTargetKnowledge::{Exact, Open}` keeps target completeness separate
from heap shape. Joins retain known target IDs and stay `Exact` only when both
inputs are exact. Only complete target knowledge permits exhaustive static
dispatch or single-target specialization; possible `Open` targets still
contribute conservative effects and capability accounting. `OptionalCallSource`
keeps syntax ownership for new optional calls and marks already-accounted
prefixes so reanalysis does not duplicate diagnostics. These callable facts do
not replace direct eval's runtime check against the original realm intrinsic.

`DynamicSourceIntrinsic` is the catalog behind the Function-family and
realm-eval identities. Derived function object prototypes expose their actual
constructor identity, and the Test262-only realm-eval builtin is admitted by
`HostSurfacePolicy::Test262`. Its defining realm determines Script execution.
Identifier spelling, a matching source string or a replaced host property
cannot manufacture intrinsic authority. The former compile-time
`DirectEvalCallSite` and erased-target classifier have been removed; direct
calls now retain the evaluated Reference and callable across argument effects.

When an actual runtime intrinsic has no matching compiled source/context, it
rejects through `lila_host.reject_dynamic_source(i64)`. The separate
`DynamicSourceRuntimeOperation` survives Wasmtime, engine and Test262
boundaries, including agent execution. It is not a JavaScript throw and cannot
satisfy `catch` or a runtime-negative exception expectation. Actual JavaScript
throws, worker failures, traps and timeouts retain their own causes; a source
gap must not hide a simultaneous Bug or Crash. User replacements and runtime
non-String eval calls retain their ordinary behavior. The import receives an
operation code and performs no source compilation.

### Historical capability-only ownership checkpoints

The following checks preceded prepared-source execution. Their exact witness
counts and implementation descriptions are retained for traceability, not as
new verification results or current restrictions on supported source forms.

- The original spread-free intrinsic `.call` route used the closed
  `DynamicSourceCallAdmission` to reject gaps before forwarded receiver,
  parameter or caller-flow observations. It preserved no-source eval result
  precision and considered retained `Exact` and `Open` targets. At that point
  `apply`, Reflect forwarding, bound functions and proxies were still listed as
  forwarding debt. `forwarded_dynamic_source_call_structure.rs` and
  `forwarded_dynamic_source_call.rs` recorded the admission and effect-order
  witnesses; prepared candidate discovery subsequently expanded those paths.
- The capability classifier's non-`Clone`, non-`Copy` `DynamicSourceProof` had
  two rows and seven lexical type mentions. Its syntax producer admitted string literals,
  no-substitution templates, parentheses and pure literal concatenations, and
  its sole exhaustive gap projection selected runtime-compilation or AOT-known
  environment debt. Merely folded IR strings could not manufacture that proof.
  The recorded `dynamic_source_proof_structure.rs` guard passed `3/3`, and the
  closed-operation/requirement witness passed `1/1`. Those results established
  a source-equivalent capability closure for diagnostic ownership before the
  finite candidate registry supplied separate guarded execution support.
- Optional-chain source ownership used the private, capability-free
  `OptionalCallSource` domain, combining call effect tokens and retaining
  already-accounted prefixes. Its structure guard passed `4/4`; the new-call
  pass-through and grouped-prefix no-duplicate-diagnostic witnesses each passed
  `1/1`. The capability contract records their commands and semantic owners.
- The private, non-`Clone`, non-`Copy` `UnsupportedDynamicSourceCall` couples
  builtin-accounting identity and `DynamicSourceGap`; its sole recorder decomposes
  that authority. The recorded structure guard pinned its sole producer/projection,
  six diagnostic-recording call sites and the already-accounted discard route.
  That source-equivalent unsupported-accounting closure removed redundant
  construct metadata without changing diagnostics, builtin counts, IR or
  capability claims. It was an accounting checkpoint, not textual source
  implementation.

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
  argument exactly once in source order. Textual invocations select a matching
  prepared unit or report their typed source/context gap.
- Finite Function candidates preserve live argument conversion, separate
  parameter/body grammars, fresh function identity and constructor realms;
  exceeding discovery bounds or missing the runtime tuple cannot create a pass.
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
cargo test --release --locked -j2 -p lila-ir \
  --test prepared_dynamic_function --test prepared_script --test direct_eval_environment
LILA_MODULE_MEMORY_CACHE_ENTRIES=1 cargo test --release --locked -j2 -p lila-engine \
  --test aot_realm_modules \
  -- aot_prepared_dynamic_function:: aot_prepared_script:: aot_direct_eval:: aot_direct_eval_call_identity:: aot_direct_eval_environment:: aot_direct_eval_escaped_arrows:: aot_dynamic_source_capability:: --test-threads=2
```

Run real filters under `built-ins/eval`, `built-ins/Function`, generator/async
function constructors, direct/indirect eval language tests and `$262.evalScript`
cross-realm cases. Report unsupported counts separately until resolved. These
are refresh instructions, not a claim that this documentation edit ran them;
completed checkpoint results belong to the repair notes and publisher-owned
status artifacts.

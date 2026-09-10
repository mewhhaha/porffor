# T13 — Dynamic source evaluation: `eval`, `Function` and realm evaluation

**Status:** Finite prepared Function-family sources and direct/indirect eval and realm Scripts implemented; unmatched runtime source remains typed Wasm-AOT debt, with the repair-cohort replay verified

**Parallel group:** Feature lane; architecture decision recorded
**Depends on:** T03, T06, T08, T09, T12  
**Blocks:** Honest accounting for dynamic-code Test262 cases and parts of T24/T26

## Current repository state

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
`EvalPassThrough`, `FunctionInvocation`, `CompiledScript` or `Unsupported`.
Known Script sources register an executable or deferred-error unit instead of
being rejected solely because they contain text. Resolved Function-family
invocations retain their runtime argument conversions and guarded source
selection even when discovery cannot prepare a matching tuple. In that case,
an abrupt argument conversion remains its real JavaScript exception; successful
conversions followed by an unmatched tuple reach the typed capability gap.

`ProvenEvalPassThrough` admits no-argument calls and non-spread calls whose first
argument has a nonempty `KindSet` excluding primitive String. It retains the
ordinary indirect-call IR, so callee identity and every argument effect remain
observable. String-capable arguments require prepared-source dispatch or a
reported gap. A spread cannot obtain that compile-time pass-through proof;
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
  --test aot_prepared_dynamic_function --test aot_prepared_script \
  --test aot_direct_eval --test aot_direct_eval_call_identity \
  --test aot_direct_eval_environment --test aot_direct_eval_escaped_arrows \
  --test aot_dynamic_source_capability -- --test-threads=2
```

Run real filters under `built-ins/eval`, `built-ins/Function`, generator/async
function constructors, direct/indirect eval language tests and `$262.evalScript`
cross-realm cases. Report unsupported counts separately until resolved. These
are refresh instructions, not a claim that this documentation edit ran them;
completed checkpoint results belong to the repair notes and publisher-owned
status artifacts.

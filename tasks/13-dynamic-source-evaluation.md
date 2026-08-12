# T13 — Dynamic source evaluation: `eval`, `Function` and realm evaluation

**Status:** Policy and typed compiler accounting implemented; static subsets remain

**Parallel group:** Feature lane; architecture decision recorded
**Depends on:** T03, T06, T08, T09, T12  
**Blocks:** Honest accounting for dynamic-code Test262 cases and parts of T24/T26

## Current repository state

The active product policy is explicit: generic `eval`, Function-family
construction and realm `evalScript` remain visible Wasm-AOT unsupported cases
when support would require an interpreter or runtime parser. Resolved ordinary
`eval` and `%Function%` calls now carry a closed `UnsupportedFeature` through
IR diagnostics into conformance accounting. The three derived Function-family
constructors now have closed compiler-only intrinsic identities carried by
function prototype shapes, and `$262.evalScript` is a typed host
identity admitted solely by the Test262 host-surface policy. Test262 no longer
infers any dynamic-source result from source spelling. The README reports all
of these cases separately.
Supported statically known subsets have not been implemented. Keep this task
focused on capability reporting and general compilation paths rather than
treating the permitted unsupported result as a pass.

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

Current compiler producers cover the resolved direct call/construct paths for
direct/indirect `%eval%`, all four Function-family constructors and realm
`evalScript`, including spread calls, optional eval and zero-argument Function
construction. The old zero-argument shortcut did not compile an empty
function; it manufactured a value with Function-constructor metadata, so it is
now typed unsupported with every other Function-constructor call.

This boundary deliberately does not claim the static subset. A literal eval
string is recorded as blocked on the caller/realm environment seam rather than
sent through Script parsing, and a Function-family literal is recorded as
blocked on the target-realm environment seam rather than synthesized as a
wrapper. Generic source remains blocked on runtime compilation.

`DynamicSourceIntrinsic` is the non-executable catalog behind the remaining
identities. Generator, async and async-generator function object shapes expose
the right constructor through their intrinsic prototype; aliases therefore
retain identity without recognizing identifier spelling. The Wasm-AOT Test262
harness stores the Test262-only realm-eval host builtin directly on
`$262.evalScript`, so lowering sees the caller's actual argument expressions.
The compiler-only Function identities have no backend emitter. Realm eval has a
defensive host body so the always-loaded harness can carry a valid function
object, but every resolved invocation produces the typed diagnostic and is
rejected before backend planning.

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

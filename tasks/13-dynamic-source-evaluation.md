# T13 — Dynamic source evaluation: `eval`, `Function` and realm evaluation

**Status:** Policy selected; in progress — explicit unsupported accounting exists, ADR remains

**Parallel group:** Feature lane with an architecture decision first  
**Depends on:** T03, T06, T08, T09, T12  
**Blocks:** Honest accounting for dynamic-code Test262 cases and parts of T24/T26

## Current repository state

The active product policy is explicit: generic `eval`, Function-family
construction and realm `evalScript` remain visible Wasm-AOT unsupported cases
when support would require an interpreter or runtime parser. The harness
classifies these cases and the README reports them separately. A dedicated ADR
comparing the allowed designs is still absent, and supported statically known
source/host-compiler subsets have not been implemented. Keep this task focused
on architecture, capability reporting and general compilation paths rather
than treating the permitted unsupported result as a pass.

## Objective

Resolve dynamic JavaScript source evaluation without violating the project ban on shipping an interpreter/VM inside emitted Wasm. Implement every compliant subset that can remain direct compilation, and report the rest explicitly until a deliberately approved host-compiler design exists.

Dynamic `import()` is explicitly not in this task's unsupported bucket: T12's componentized-AOT strategy handles it by resolving specifiers to precompiled module components at runtime. This task covers only textual dynamic source — `eval`, the `Function`-family constructors and realm `evalScript`.

## Required architecture decision record

Before implementation, write an ADR comparing these options:

1. **AOT-known source only:** compile direct `eval`/Function bodies whose source is statically known, while preserving direct-eval scope semantics.
2. **Optional Rust host compiler service:** Wasm requests compilation/execution from the embedding Rust engine, with explicit capability negotiation and shared realm/state bridging.
3. **Explicit product unsupported:** keep dynamic source visible as unsupported for Wasm-AOT. The spec-exec oracle may still execute these cases during differential triage, but an oracle pass never counts as product support.

Reject compiling a generic parser/interpreter into the Wasm artifact and reject test-specific source recognition. The ADR must address standalone artifacts, security, CSP-like policies, deterministic builds, scope capture, realm ownership, heap identity and re-entrancy.

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

## Host compiler service requirements, if selected

- Typed host import rather than a magic `eval` opcode.
- Compile source with the same Rust front/IR/Wasm pipeline.
- Define a state bridge so evaluated code sees and mutates the required environment/realm objects without copying observable identity.
- Cache only when source, realm policy and environment shape make caching unobservable.
- Prevent recursive compilation from corrupting the active Wasm instance.
- Expose a clear error when the embedding host disables dynamic compilation.

## Acceptance criteria

- The repository has one documented policy; no ambiguous fallback.
- Supported static direct-eval cases preserve lexical scope and abrupt completions.
- Indirect/cross-realm evaluation never aliases the wrong global.
- Unsupported dynamic cases are classified consistently and remain in real-suite accounting.
- No source regex/materialization exists for known Test262 eval/Function cases.
- If a host service is implemented, representative dynamic strings—not known at AOT time—pass scope, realm, constructor and error tests.
- The README/CLI clearly report artifact capability requirements.

## Required tests

```sh
cargo test -p porffor-front eval_ --quiet
cargo test -p porffor-ir eval_ --quiet
cargo test -p porffor-engine eval_ --quiet
cargo test -p porffor-cli eval_ --quiet
```

Run real filters under `built-ins/eval`, `built-ins/Function`, generator/async function constructors, direct/indirect eval language tests and `$262.evalScript` cross-realm cases. Report unsupported counts separately until resolved.

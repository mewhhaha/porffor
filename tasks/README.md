# Porffor Rust AOT + Test262 execution plan

This directory is the implementation backlog for the Rust rewrite. It is designed so multiple agents can work concurrently without turning `crates/porffor-ir/src/lib.rs` and `crates/porffor-aot-wasm/src/lib.rs` into permanent merge-conflict bottlenecks.

The north star is the repository contract in `AGENTS.md`: Porffor compiles JavaScript directly to Wasm, does not ship an interpreter/VM inside the artifact, and drives the pinned real Test262 suite to zero unowned failures. Fake-suite results are smoke tests only. An `Unsupported` result is visible debt, never a passing result, and must not be hidden in a skip list or status denominator.

## Non-negotiable rules

1. Product execution remains `parse -> early errors -> spec IR -> lowering IR -> Wasm codegen`.
2. Do not add source-path, test-name, or assertion-text branches that manufacture Test262 results. Existing focused materializations must be catalogued and retired as general semantics replace them.
3. Every change starts with a reproducible failing real Test262 filter or exact case and ends with the same command green. Add a small CLI/engine regression fixture when it isolates the behavior better than the upstream case.
4. Preserve evaluation order, abrupt completion, realm ownership, property attributes, observable coercions, and proxy traps. Passing the happy path is not enough.
5. Do not hand-edit published conformance totals. Use `porf test262 publish-status` or `scripts/publish-real-status-low-ram.sh` after a complete verified matrix.
6. Keep `unsafe_code = "forbid"`. New dependencies require a reason, license review, deterministic behavior, and a clear Wasm/runtime story.
7. Feature PRs should not combine unrelated refactors. When a prerequisite interface is missing, land the interface first under its foundation task.

## How to execute one task

1. Read this file, the selected task file, `AGENTS.md`, and the touched crate manifests.
2. Record the exact baseline commands and counts in the PR description.
3. Implement the smallest general semantic layer that fixes the whole failure family; do not special-case the representative test.
4. Run `cargo fmt --all --check`, targeted crate tests, focused CLI fixtures, and the real Test262 filter listed by the task.
5. Search for regressions in adjacent filters that share the same abstract operation or builtin.
6. In the PR description, report: files changed, semantic invariant added, exact tests/counts, remaining failures, and follow-up task IDs.

## Parallel work graph

### Bootstrap and coordination

These can begin immediately. `T02` should land early because it creates merge-friendly ownership boundaries.

| ID | Task | Parallel notes |
|---|---|---|
| [T00](00-operating-contract.md) | Operating contract and contribution protocol | Documentation/CI only; independent |
| [T01](01-baseline-and-generated-backlog.md) | Reproducible real-suite baseline and generated failure inventory | Independent; feeds every lane |
| [T02](02-modularize-ir-and-wasm-backend.md) | Split monolithic IR/backend modules | Coordinate before broad feature work |
| [T03](03-conformance-harness-integrity.md) | Honest Test262 harness and host contract | Independent of most language semantics |
| [T04](04-spec-operations-and-completion-abi.md) | Shared abstract operations and completion ABI | Foundation for most feature lanes |

### Core semantic foundations

Run these in parallel after the relevant portions of `T02`/`T04` are stable.

| ID | Task | Primary ownership |
|---|---|---|
| [T05](05-values-heap-gc.md) | Value representation, heap, GC, weak reachability | runtime + Wasm heap modules |
| [T06](06-realms-intrinsics-cross-realm.md) | Realms, intrinsics, host hooks, cross-realm identity | runtime + intrinsic bootstrap |
| [T07](07-parser-grammar-early-errors.md) | Parser boundary, grammar coverage, early errors | front + IR parser boundary |
| [T08](08-environments-control-flow.md) | Environments, TDZ, references, control flow and abrupt completion | IR lowering + control-flow emitter |
| [T09](09-functions-classes-private-elements.md) | Call/construct, functions, classes, private elements | function/class lowering and emitter |
| [T10](10-object-model-descriptors-exotics.md) | Ordinary objects, descriptors, integrity, exotic object protocol | object/descriptor modules |

### Feature lanes

Once their listed foundations are present, these lanes should be owned by separate agents and merged independently.

| ID | Task | Depends on |
|---|---|---|
| [T11](11-proxy-reflect-metaobject.md) | Proxy and Reflect meta-object protocol | T04, T05, T06, T10 |
| [T12](12-modules-linking-loading.md) | Modules, linking, namespace objects, TLA host flow | T06, T07, T08, T09, T10 |
| [T13](13-dynamic-source-evaluation.md) | `eval`, `Function`, realm evaluation policy/implementation | T06, T08, T09, T12 |
| [T14](14-promises-jobs-async.md) | Promise jobs, async functions, async iteration | T04, T05, T06, T09 |
| [T15](15-generators-iterators-resource-management.md) | Generators, iterators/helpers, disposal | T04, T05, T08, T09 |
| [T16](16-arrays-and-array-builtins.md) | Array exotic semantics and complete Array API | T04, T05, T10 |
| [T17](17-typedarrays-binary-data-atomics.md) | ArrayBuffer, DataView, typed arrays, SAB, Atomics | T04, T05, T06, T10; async wait paths use T14 |
| [T18](18-strings-unicode.md) | ECMAScript strings and Unicode-correct String API | T04, T05, T10 |
| [T19](19-regexp.md) | ECMAScript RegExp syntax and semantics | T04, T05, T10, T18 |
| [T20](20-number-bigint-math-json.md) | Numeric semantics, BigInt, Math, JSON | T04, T05, T10 |
| [T21](21-symbols-collections-weakrefs.md) | Symbols, Map/Set, weak collections, WeakRef/finalization | T05, T06, T10 |
| [T22](22-date-temporal.md) | Date and Temporal | T04, T05, T06, T10, T18, T20 |
| [T23](23-intl402.md) | ECMA-402 Intl implementation | T06, T10, T18, T20, T22 |
| [T24](24-globals-errors-annexb-host.md) | Globals, native errors, Annex B, remaining host-visible builtins | T04, T06, T07, T09, T10 |

### Validation and closure

| ID | Task | Depends on |
|---|---|---|
| [T25](25-differential-fuzzing-performance.md) | Differential testing, fuzzing, timeout and code-size work | T01-T04; runs continuously |
| [T26](26-zero-failure-conformance-closure.md) | Full pinned suite closure and release gate | All applicable tasks |

## Merge-conflict policy

Until `T02` lands, only one active PR should make broad edits to either giant `lib.rs`. Other agents should work in tests, harness code, runtime code, or narrowly isolated functions. After modularization, each feature lane must own a dedicated IR module, Wasm emitter module, builtin module, and focused fixture prefix. Shared ABI changes belong in `T04` and should land before dependent feature PRs.

When two tasks require the same abstract operation, the first agent implements it in the shared operation layer with unit tests; the second consumes it. Do not copy slightly different `ToObject`, `ToLength`, `Get`, `Call`, iterator, descriptor, or completion logic into feature-specific code.

## Definition of done for a feature lane

A lane is complete only when:

- its real Test262 subtree is fully green for the intended backend and pinned revision;
- parser, early-error, runtime, backend, host-harness, timeout, and crash failures are all zero in that subtree;
- no test-specific semantic materialization remains for the covered behavior;
- descriptor metadata, subclassing/species, proxies, cross-realm behavior, abrupt completions, and coercion order have representative coverage;
- adjacent fake-suite and CLI regression tests remain green;
- README status is refreshed only if a complete real-suite publication was performed.

## Final acceptance target

`T26` owns the final evidence: a complete resumable matrix for the current pin, verified snapshot artifacts, zero crashes and bugs, no silent skips, no stale status claims, and an explicit accounting of any dynamic-source cases permitted by `AGENTS.md`. Literal `passed == total` remains the project target; architecture exceptions must stay separately visible until the project deliberately resolves them.
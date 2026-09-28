# Conformance ownership domains

`T00`–`T29` are stable compiler and conformance domain identifiers. They appear
in the closed `lila_ir::TaskId` enum, diagnostics, shortcut accounting,
`test262/backlog/ownership-map.tsv`, and the CLI regression ledger. They are
domain labels, not the current repair tasks or a claim that an area is complete.
The actionable measured failures are in [tasks](../../tasks/README.md), with
separate `Fxxx` identifiers, source owners, evidence, and validation commands.

| Domain | Responsibility | Primary implementation or contract |
| --- | --- | --- |
| T00 | Contribution and verification policy | [AGENTS.md](../../AGENTS.md), [batch workflow](batch-workflow.md) |
| T01 | Baselines, execution identity, generated backlog | `lila-test262`; [snapshot comparison](test262-snapshot-comparison.md) |
| T02 | Compiler module boundaries and shared ownership | `lila-ir`, `lila-aot-wasm`; `scripts/check-module-boundaries.sh` |
| T03 | Test262 harness and host integrity | `lila-test262`; [taxonomy](conformance-taxonomy.md) |
| T04 | Spec operations and completion ABI | `lila-ir`; [operation descriptors](operation-descriptors.md) |
| T05 | Values, heap, GC, weak reachability | `lila-aot-wasm`, `lila-engine`; [heap architecture](value-heap-gc.md) |
| T06 | Realms and intrinsic identity | [Realm intrinsics](realm-intrinsics.md) |
| T07 | Parsing, grammar and early errors | `lila-front`, `lila-ir` |
| T08 | Environments, References and control flow | `lila-ir`, `lila-aot-wasm` |
| T09 | Functions, constructors, classes and private elements | `lila-ir`, `lila-aot-wasm` |
| T10 | Objects, descriptors and exotic objects | `lila-aot-wasm::objects` |
| T11 | Proxy and Reflect | `lila-aot-wasm::objects`, `builtins::reflect` |
| T12 | Modules, linking, loading and namespaces | `lila-ir`, `lila-aot-wasm::modules`, `lila-engine` |
| T13 | Prepared sources and the dynamic code generation boundary | `lila-ir`, `lila-aot-wasm::functions`, `lila-engine` |
| T14 | Promises, jobs, async functions and iteration | `lila-aot-wasm`, `lila-engine` |
| T15 | Generators, iterators and resource management | `lila-ir`, `lila-aot-wasm` |
| T16 | Arrays and Array builtins | `lila-aot-wasm::builtins::array` |
| T17 | Typed arrays, buffers, DataView, shared memory and Atomics | `lila-aot-wasm` |
| T18 | Strings and Unicode | `lila-ir`, `lila-aot-wasm::builtins::string` |
| T19 | RegExp | [RegExp architecture](regexp-engine.md) |
| T20 | Number, BigInt, Math and JSON | `lila-ir`, `lila-aot-wasm` |
| T21 | Symbols, collections and weak references | `lila-aot-wasm`, `lila-engine` |
| T22 | Date and Temporal | `lila-aot-wasm`, `lila-intl` |
| T23 | ECMA-402 Intl and deterministic locale data | `lila-intl`; [Intl architecture](intl-architecture.md) |
| T24 | Globals, errors, Annex B and host builtins | `lila-front`, `lila-aot-wasm::builtins::host` |
| T25 | Differential testing, fuzzing and performance | `lila-test262`, `lila-engine` |
| T26 | Complete conformance evidence and release gate | `lila-test262`; [publication contract](reproducible-publication-driver.md) |
| T27 | Interpreter quarantine and Wasm-AOT product default | [Architecture invariants](architecture-invariants.md) |
| T28 | Retired JavaScript product boundary | [AGENTS.md](../../AGENTS.md) |
| T29 | Canonical Lila identifiers | [Identity contract](lila-identity-migration.md) |

The prefix map selects a domain; it does not establish a root cause. A failure
task must separately distinguish a confirmed cause from a hypothesis and cite
the affected execution identities. `T26-unclassified` is an explicit triage
bucket, not a member of the closed domain enum or a resolved owner.

Only complete verified pinned Wasm-AOT evidence can update product conformance
status. Fake suites, focused replays, partial checkpoints and `spec-exec`
diagnostics have separate roles. The Rust publisher owns the canonical
JSON/text publication pair and generated README block; the status-artifact
guard enforces that boundary. Unsupported dynamic code generation, crashes,
bugs and timeouts remain visible non-passing outcomes.

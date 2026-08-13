# T03 — Test262 harness integrity and host contract

**Status:** In progress — host contracts and an exact typed shortcut ledger landed; semantic cleanup remains

**Parallel group:** Bootstrap/foundation  
**Depends on:** T01 for the authoritative inventory  
**Blocks:** Trustworthy results for every feature lane

## Current repository state

The repository has a checked-in host-ABI contract, shortcut inventory, exact
per-entry ledger and CI audits. `./scripts/check-test262-host-abi.sh` passes in
the current working tree. `./scripts/audit-test262-shortcuts.sh --check` now
pins every production observation on its audited surface by a stable key and
SHA-256 source fingerprint. Rewrite-dispatch entries are keyed by the called
rewrite function, so deleting one no longer renumbers every later entry;
observations inside a declaration retain a local occurrence ordinal. It rejects new, missing,
duplicated or drifted entries, invalid classifications and non-concrete task
IDs, then byte-compares the generated inventory.

The current ledger contains 440 observations: 32 legitimate harness
adaptations, 55 diagnostic instrumentation sites and 353 semantic shortcuts.
Every entry has a concrete owner, removal task and closed reason code; none use
`T26-unclassified`. This is an honest cleanup map, not completion. The semantic
materialization layer is still large, so harness results cannot yet satisfy
this task's integrity acceptance criteria.

The Wasm `agent_call` transport no longer duplicates raw operation integers
between the AOT emitter and engine. `lila-runtime::AgentHostOperation` is the
single closed 13-operation wire domain: the emitter writes its explicitly
pinned `i64` values, the engine rejects an unknown word once at the import
boundary, and the semantic dispatch is an exhaustive Rust match. This closes
one host-ABI drift path; it does not establish that every `$262` operation or
agent case satisfies the acceptance criteria below.

Direct `test262 run` and `test262 shard` completion now cross one typed verdict
boundary. `NoEvidence`, a non-empty all-pass `Passed`, and a non-empty
`Failed` verdict are distinct states backed by `NonZeroUsize`; inconsistent
total, pass and failure counts do not produce a verdict. The CLI exhaustively
maps only `Passed` to process success, while retaining a failed run's snapshot
before returning a non-zero exit. This closes command-level false-green and
zero-selection paths; it does not prove that the harness semantics which
produced a verdict are correct.

## Objective

Make the Test262 runner an honest observer of compiler behavior rather than a second semantic implementation. Replace source-pattern simulations, test-path materializations and permissive host fallbacks with explicit host APIs and general compiler/runtime semantics.

Current areas to audit include the embedded local-harness assets owned by
`lila-test262`, source materialization, `RunOptions.test_path`, and Wasm
backend branches that recognize exact Test262 paths or source shapes.

## Work items

### 1. Inventory semantic shortcuts

Generate a checked-in report of every branch that depends on:

- an exact test path or directory;
- assertion text, source regexes or known helper source;
- a hard-coded expected value for a real Test262 case;
- replacing an upstream helper with reduced behavior;
- converting a timeout into success.

Classify each item as legitimate harness adaptation, temporary diagnostic instrumentation, or semantic cheat. Assign removal to the relevant task ID.

The checked-in ledger completes this classification baseline for the current
mechanical scan. Stable keys intentionally exclude line numbers; line numbers
exist only in the generated report for review. Any source edit that retains a
key but changes its matched expression changes the fingerprint and fails the
guard. Semantic entries remain open work in their removal tasks.

### 2. Define the `$262` host ABI

Specify typed host operations for at least:

- `global`, `getGlobal`, `createRealm`, realm `evalScript` and `destroy`;
- `detachArrayBuffer`;
- `gc`;
- `IsHTMLDDA` and `AbstractModuleSource` where required by the pin;
- agent start/broadcast/report/sleep/leaving/monotonic time;
- async completion and `$DONE` reporting.

The Wasm-AOT product runner owns this ABI. The spec-exec oracle runner may implement it differently for differential runs, but the JavaScript-visible behavior and failure reporting must match, and nothing about the ABI design may assume the interpreter is available on the product path.

### 3. Remove fake concurrency behavior

The local harness currently contains source-pattern handling and `new Function`-based agent simulation. Replace this with real host-managed agents, shared backing stores and waiter queues. Never parse agent source with regexes to infer its expected behavior.

### 4. Separate harness adaptation from product semantics

Prelude merging may adapt Test262's shell contract, but it must not implement missing Array, Atomics, Promise, Proxy or other ECMAScript semantics. Product builtins must be installed by the runtime/compiler path.

### 5. Add integrity checks

Add tests or a lint that reject new exact-path semantic branches outside a narrowly documented allowlist for discovery/snapshot routing. The allowlist must name the reason and removal task.

## Failure behavior

Missing host capability must produce a stable `HostHarness` or explicit `Unsupported` failure. It must not return an object that aliases the current realm, silently ignore buffer detachment, or synthesize an agent report that lets a test pass.

## Acceptance criteria

- The harness contains no source regexes that emulate agent programs or expected assertions.
- Every `$262` method has a documented backend contract and direct tests.
- Realms are distinct or the operation fails explicitly; no same-global fallback.
- Buffer detachment and GC hooks either perform the requested operation or fail visibly.
- The runner correctly handles async pass, async rejection, timeout and duplicate `$DONE`.
- The generated shortcut inventory has an owner and removal task for every remaining item.
- Running an intentionally unsupported host case cannot be counted as success.

## Required tests

```sh
cargo test -p lila-spec-exec --quiet
cargo test -p lila-test262 --quiet
cargo test -p lila-engine --quiet
cargo test -p lila-cli test262_ --quiet
./target/debug/lila test262 run harness --execution-backend wasm
# Oracle runner parity check (diagnostic only):
./target/debug/lila test262 run harness --execution-backend spec
```

Add focused fake fixtures for each host method, but validate representative real `harness`, `language/module-code`, `built-ins/Atomics` and cross-realm cases before completion.

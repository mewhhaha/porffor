# T03 — Test262 harness integrity and host contract

**Status:** Ready  
**Parallel group:** Bootstrap/foundation  
**Depends on:** T01 for the authoritative inventory  
**Blocks:** Trustworthy results for every feature lane

## Objective

Make the Test262 runner an honest observer of compiler behavior rather than a second semantic implementation. Replace source-pattern simulations, test-path materializations and permissive host fallbacks with explicit host APIs and general compiler/runtime semantics.

Current areas to audit include `test262/harness.js`, `porffor-test262` source materialization, `RunOptions.test_path`, and Wasm backend branches that recognize exact Test262 paths or source shapes.

## Work items

### 1. Inventory semantic shortcuts

Generate a checked-in report of every branch that depends on:

- an exact test path or directory;
- assertion text, source regexes or known helper source;
- a hard-coded expected value for a real Test262 case;
- replacing an upstream helper with reduced behavior;
- converting a timeout into success.

Classify each item as legitimate harness adaptation, temporary diagnostic instrumentation, or semantic cheat. Assign removal to the relevant task ID.

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
cargo test -p porffor-spec-exec --quiet
cargo test -p porffor-test262 --quiet
cargo test -p porffor-engine --quiet
cargo test -p porffor-cli test262_ --quiet
./target/debug/porf test262 run harness --execution-backend wasm
# Oracle runner parity check (diagnostic only):
./target/debug/porf test262 run harness --execution-backend spec
```

Add focused fake fixtures for each host method, but validate representative real `harness`, `language/module-code`, `built-ins/Atomics` and cross-realm cases before completion.
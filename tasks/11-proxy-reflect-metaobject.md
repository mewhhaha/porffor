# T11 — Proxy and Reflect meta-object protocol

**Status:** In progress — Proxy/Reflect paths exist; materialized cases and invariants remain

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T06, T09, T10  
**Blocks:** Proxy-sensitive closure in most other lanes

## Current repository state

Proxy and Reflect builtins are implemented through dedicated backend paths, and
focused tests cover several traps and object-integrity interactions. The
Test262 materializer still contains Proxy-specific exact-path rewrites,
including creation, revocation, call/construct and descriptor traps. Until
those rewrites are removed and the complete Proxy/Reflect trees are verified,
this lane remains open.

`lowering/proxy_traps.rs` now owns one private, closed `ProxyTrap` domain
containing all thirteen ECMA-262 10.5 handler methods. Each trap maps
exhaustively to one of eight semantic argument records rather than to an
untyped arity. When a proven
`new Proxy(target, handler)` path has a statically visible handler method,
pre-lowering and typed lowering both enumerate every trap through that mapping.
Ordinary object-literal lowering deliberately retains its former five-name
heuristic so methods merely named `apply`, `construct`, or `set` are not
misclassified as traps. This removes the former raw-string match and its
catch-all, which silently discarded eight valid trap signatures. The seam is
covered by the green central feature-enabled CLI compile; focused Proxy
execution remains unverified. It is an inference invariant, not a claim that
the runtime implementations or full Proxy/Reflect trees are complete.

## Objective

Implement every Proxy internal method and every Reflect method through the shared object/call protocols, including revocation and all invariant checks. Remove static-shape behavior that bypasses observable traps.

## Proxy scope

Support proxies over ordinary, callable, constructable and exotic targets. Implement:

- creation validation and `Proxy.revocable`;
- revocation behavior for every internal method;
- `getPrototypeOf`, `setPrototypeOf`, `isExtensible`, `preventExtensions`;
- `getOwnPropertyDescriptor`, `defineProperty`, `has`, `get`, `set`, `deleteProperty`, `ownKeys`;
- `apply` and `construct`;
- nested proxies and proxies as handlers/targets;
- realm-correct errors and target/handler lifetime.

Each trap must use `GetMethod`, invoke with the correct handler `this`, preserve argument order, and fall back to the target's real internal method when absent.

## Invariant checks

Implement all post-trap checks, including:

- non-configurable and non-existent property constraints;
- non-writable data/accessor consistency;
- non-extensible target restrictions;
- prototype equality requirements;
- `ownKeys` duplicate/type checks and exact inclusion constraints;
- callable/constructable target requirements;
- object-result requirements for descriptor/prototype/construct traps.

Do not weaken invariants for arrays, typed arrays, module namespaces or other exotic targets.

## Reflect scope

Complete all Reflect methods and route them to shared operations:

- `apply`, `construct`;
- `defineProperty`, `deleteProperty`;
- `get`, `set`, `has`;
- `getOwnPropertyDescriptor`, `getPrototypeOf`, `setPrototypeOf`;
- `isExtensible`, `preventExtensions`, `ownKeys`.

Reflect methods return booleans where specified rather than throwing on ordinary failure, while still propagating abrupt completions from coercion/traps.

## Acceptance criteria

- The full pinned `built-ins/Proxy` and `built-ins/Reflect` trees pass.
- Every trap has unit tests for absent trap, successful trap, thrown trap and invariant violation.
- Revoked proxies fail consistently for every operation.
- Proxy-wrapped functions/classes preserve call/construct/new-target behavior.
- Proxy operations work against arrays, typed arrays and non-extensible targets.
- No property/call fast path skips a possible proxy trap without a proven non-proxy guard.
- Nested proxy and cross-realm handler tests pass without materialization.

## Required tests

```sh
cargo test -p lila-aot-wasm proxy_ --quiet
cargo test -p lila-cli proxy_ --quiet
./target/debug/lila test262 run built-ins/Proxy --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Reflect --execution-backend wasm-aot --timeout-ms 120000 --threads 4
```

Re-run adjacent Object, Array, TypedArray and Function filters because proxy invariants are shared across them.

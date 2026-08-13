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
covered by the green central feature-enabled CLI compile and by the complete
620-test default CLI inventory, including focused apply, construct,
defineProperty and set behavior. It is an inference invariant, not a claim
that the runtime implementations or full Proxy/Reflect trees are complete.

The Wasm-AOT `has` path now shares T10's closed `[[HasProperty]]` dispatcher.
An absent `has` trap re-enters the complete target dispatch rather than a
representation-specific fallback, including through nested Proxy targets, and
any callable value is accepted as the trap, including a callable Proxy. The
positive regression is dry-written but its focused runtime gate has not run
while the shared conformance matrix is active.

The bounded representation now retains both `[[ProxyTarget]]` and
`[[ProxyHandler]]` as typed payload/tag pairs. A single Proxy allocator takes
`ProxySlotLocals` with distinct target and handler newtypes; both constructors
must supply it, while its slot writer is private. Omission or target/handler
transposition is therefore a compile error.
Revocation keeps the existing handler-payload
sentinel and does not change target layout. The `has` consumer then loads the
retained tag and routes lookup through the existing full object-read seam,
with abrupt getter completion leaving the traversal and the exact tagged
handler passed as trap `this`. Other Proxy methods that reconstruct Object
remain separate T11 migrations over the same stored slot. The exact Wasm-AOT
regression for Function, Array, arguments and nested-Proxy handlers, tagged
handler `this`, and an abrupt lookup getter is written but has not run.

The post-trap false-result check remains incomplete and this task stays open:
it still scans ordinary Object/Function descriptor storage directly instead of
calling typed `[[GetOwnProperty]]` and `[[IsExtensible]]` dispatch. Therefore an
Array target's non-configurable `length` (and corresponding exotic cases) can
still evade the required invariant: a false `has` trap for `length` must throw,
but the current checker can return `false`. No Array-only special case or
expected failure was added; the durable repair is the shared typed descriptor
and extensibility seam. The public descriptor builtin does cover Array,
arguments and integer-indexed storage, but calling that allocating builtin
from inside the raw HasProperty branch would couple completion routing and
allocation into this dispatcher. The next bounded seam should instead expose
a value-free typed own-descriptor fact emitter shared by that builtin and
Proxy invariant checks; the existing compact helper is Array-specific and is
not duplicated here.

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

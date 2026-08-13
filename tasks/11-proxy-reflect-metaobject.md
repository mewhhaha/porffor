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

The post-trap boolean-result batch now has one direct-target contract. A
private, closed object-representation order is shared by `[[HasProperty]]` and
a value-free own-descriptor fact emitter. The fact is two distinct Wasm locals,
`present` and `descriptor`: zero is a legal descriptor word for a present data
property whose three attributes are false, so it may never double as the
absence sentinel. Its fields are private and consumers can ask only the named
presence/configurability/writability questions, making a raw-local
transposition a compile error. The emitter allocates no JavaScript descriptor
object and invokes no property getter.

That direct emitter owns integer-indexed, Array, arguments, boxed-String,
Function-special and ordinary storage in the same exhaustive representation
match. In particular, Array `length` is an unconditional present,
non-configurable descriptor; arguments `length` carries an explicit own-property
bit while it is live, including the all-false data-descriptor state; and an
invalid canonical integer-index is handled as absent rather than falling
through to ordinary storage. The public descriptor builtin's Proxy target
checks, the `has` false-result check and the `deleteProperty` true-result check
all consume this fact. The two boolean-trap invariants accept an absent
descriptor, reject a present non-configurable descriptor, and only then call
the shared typed `[[IsExtensible]]` emitter for a present configurable
descriptor. This preserves ECMA-262 order and the distinct Array/arguments
extensibility slots. The former Array-only `has` mirror and the raw
Array/ordinary delete scans are deleted.

The exact Wasm-AOT regression is written for Array `length`, both ordinary and
non-configurable Array indices, a configurable named property on a
non-extensible Array, a present ordinary all-false descriptor word,
boxed-String indices, a fixed typed-array index, ordinary arguments indices,
arguments `length` (including delete-and-recreate), both public descriptor-trap
result forms and the absent/non-extensible ordering case. It has not run while
the release matrix owns runtime verification.

The existing Proxy `deleteProperty` fixture now applies the same post-trap
contract directly to Array named and symbol properties, a configurable property
on a non-extensible Array, boxed-String virtual properties, Function
`prototype`, arguments all-false and non-configurable descriptors, and an
absent property on a non-extensible target. Trap-false and descriptor-absent
paths remain early exits, while only a present configurable property consults
the shared extensibility operation. This is a post-trap migration only: the
delete trap lookup/fallback path still has its older bounded handler dispatch,
and the direct descriptor fact does not recursively validate a nested Proxy
target. The expanded fixture has not run while the release matrix owns runtime
verification.

Proxy `[[Set]]` truthy-result validation now joins those direct-target
consumers through a richer projection of the same descriptor authority. One
closed Rust result domain contains the value-free fact and a complete Proxy-Set
record with distinct fact, data-value and setter locals; every projection
consumes the same exhaustive object-representation loop. Target, property key
and incoming value are typed call-site roles. Array, arguments,
boxed-String, Function-special and ordinary values are read from descriptor
storage without invoking getters, and mapped arguments data observes the
current parameter value. Missing setters normalize to tagged `undefined`; a
callable Proxy setter is accepted because ECMA-262 tests only whether
`[[Set]]` is undefined. Ordinary entry storage wins before virtual fallbacks,
so freezing a Function's materialized `prototype` entry changes the invariant
while DataView/intrinsic fallbacks remain available when no entry exists.

The focused Wasm-AOT fixture covers Array length and dense/sparse indices,
named and Symbol keys, boxed-String virtual values, mapped arguments and an
arguments accessor whose getter must not run, callable-Proxy setters,
`SameValue` edge cases, writable and frozen Function `prototype` entries,
integer-indexed no-false-positive cases and both assignment and Reflect entry
points. It is written but has not run while the shared verification lane owns
Cargo and Test262. This is only the post-trap, direct-target migration: Set
trap lookup/fallback and recursive nested-Proxy target `[[GetOwnProperty]]`
remain T11 work.

Proxy `[[Get]]` post-trap validation now consumes a second richer projection of
that same direct descriptor authority. The closed projection domain has
distinct Proxy-Get and Proxy-Set records, and a closed getter/setter endpoint
enum makes using the wrong accessor role an exhaustive-match type error. The
Get invariant accepts typed target, property-key and normal trap-result roles.
A trap call initially yields a distinct pending result; the only transition to
the normal-only type emits abrupt-completion routing first, so a trap's thrown
value cannot be replaced by a later frozen-target TypeError.

The shared storage-only walk observes Array dense/sparse and named entries,
Array length, mapped and accessor arguments indices, arguments special
`length`/`callee`, boxed-String virtual values, ordinary entries and
Function/DataView special values without invoking a stored getter. Missing
getters normalize both raw zero and tagged `undefined`. The invariant then
requires `SameValue` for a present non-configurable, non-writable data
descriptor and requires an undefined trap result for a present
non-configurable accessor with no getter. The former Object/Function-only raw
entry scan is deleted.

The exact Wasm-AOT fixture covers direct and Reflect Get, all of those direct
representations, callable-Proxy and missing getters without invocation,
`SameValue` edge cases, configurable/integer-indexed/absent false-positive
guards, and preservation of the original thrown trap. It is written but has
not run while the shared verification lane owns Cargo and Test262. This remains
only a direct-target post-trap migration: Get trap lookup/fallback, recursive
nested-Proxy target `[[GetOwnProperty]]`, module namespaces and complete
Proxy/Reflect Get closure remain T11 work.

The retained Proxy slots now also have one typed read authority. The reader
accepts the same `ProxySlotLocals` record as the writer, maps each heap word into
the distinct target/handler newtype, and emits the revoked-handler check before
the loaded slots become usable. Its closed completion route keeps the existing
builtin, internal-helper and HasProperty throw boundaries explicit. Both the
public descriptor path and shared `[[IsExtensible]]` now join `has` in consuming
the exact handler tag and the proxy-aware object-read seam for `GetMethod`.
Function, Array, arguments and nested-Proxy handlers therefore retain their
storage behavior and exact handler-as-`this` identity in these three methods;
an abrupt trap lookup is routed before callable/absent classification.

The exact Wasm-AOT regression covering those four handler representations,
Object and Reflect entry points, exact `this`, and abrupt lookup is written but
has not run while the release matrix owns runtime verification.

The shared proxy-aware `[[GetPrototypeOf]]` emitter now consumes that same typed
live-slot reader and full object-read seam. It no longer reconstructs every
handler as an Object, so Function, Array and arguments handlers retain their
tags for both `GetMethod` and trap `this`, while a Proxy handler observes the
complete `[[Get]]` protocol. Abrupt method lookup is routed before the
absent/non-callable split. The existing object-or-null result check,
non-extensible target prototype equality check, nested-target fallback and late
result publication remain unchanged.

The existing `getPrototypeOf` Wasm-AOT fixture now covers both Object and
Reflect entry points across those handler representations, an inherited `get`
trap on a Proxy handler's own handler and an abrupt accessor lookup. It has not
run while the release matrix owns runtime verification. Other Proxy methods
that still reconstruct an Object handler remain separate migrations.

This is deliberately not the recursive Proxy descriptor-record protocol.
When `[[ProxyTarget]]` is itself a Proxy, `[[GetOwnProperty]]` must run that
Proxy's `GetMethod`, call and full `IsCompatiblePropertyDescriptor` validation;
re-entering the allocating public builtin would violate this seam. Handler-tag
preservation in that descriptor path, nested Proxy targets, full descriptor
compatibility and module-namespace exotics therefore remain explicit T11 work.
The direct-target batch closes the Array/arguments/integer-indexed/boxed-String
and ordinary invariant gap without claiming those cases.

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

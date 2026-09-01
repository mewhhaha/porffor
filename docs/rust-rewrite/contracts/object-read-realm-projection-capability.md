# Object-read Realm projection capability

Status: the original projection was focused-verified on 2026-08-27; the
Proxy-dispatch extension was focused-verified on 2026-08-29.

## Scope

This contract owns the two internal projections from
`ObjectReadErrorRealmSource` into object-read behavior. It does not own the
source domain, function-body classification, runtime-helper catalog, Proxy
`[[Get]]`, error allocation or completion routing.

## Rust invariant

The object emitter retains two distinct private, non-derived projection
domains. `OutlinedObjectReadRealmArgument` decides whether the outlined helper
ABI argument receives the trusted current environment or zero.
`ObjectReadRevocationErrorRealm` decides whether an inlined revoked-Proxy
TypeError uses the current function's Realm or the main-Realm runtime fallback.
Combining them would erase the difference between an ABI argument and direct
error construction.

Both domains project `GlobalFallback` to `MainRealmFallback`. They project
`StandardBuiltinEnvironment`, `ObjectReadHelperArgument` and
`ProxyDispatchHelperArgument` to `TrustedCurrentEnvironment`. The two helper
sources remain distinct: only `ObjectRead` and `ObjectReadProxy` receive the
first, while only `ProxyCall` and `ProxyConstruct` receive the second. Every
helper assignment, projection and consumer is exhaustive. Neither projection
domain supports clone, copy, debug, equality or default observation; the
focused unit verifies its expected rows through exhaustive matches rather than
equality assertions.

The Proxy dispatch helpers need the added source because `GetMethod` for their
`apply` and `construct` traps uses the shared object-read operation. Their ABI
parameter 6 already contains the trusted standard-builtin environment or zero.
The object-read projection forwards that same word through `ObjectReadProxy`
and ordinary `ObjectRead`, so a revoked Proxy handler or callable Proxy
accessor creates its TypeError in the original execution Realm. No unrelated
helper becomes trusted.

The original capability cleanup changed no emitted instruction. The
Proxy-dispatch extension is behavior-bearing: it forwards the already-trusted
execution Realm through trap lookup. It changes no property-read order, trap
receiver, completion route or unrelated helper ABI.

## Verification and non-claims

At the 2026-08-27 checkpoint, the dedicated structure target passed `4/4`, the
exact projection unit passed `1/1`, the neighboring object-read Realm structure
target passed `3/3`, and the created-Realm revoked-Proxy CLI fixture passed
`1/1`. The shared formatting, compile, diff, module-boundary and task-plan
checkpoint was also green with the workspace's existing warnings.

At the 2026-08-29 Proxy execution-Realm checkpoint, the four focused and
neighboring structure targets passed `16/16`, including this target's `4/4` and
the object-read Proxy target's `3/3`. The three matching exhaustive
Realm-source projection units passed `3/3`, the exact Proxy CLI witness passed
`1/1`, and four raw Proxy leaves passed `8/8`. Formatting, the affected
all-target compile, module-boundary, task-plan, 239-entry shortcut-inventory
and diff checks were green with only the existing Boa warning.

This extension does not claim an object-model redesign, complete Proxy
conformance, a broad Test262 result, a Wasm golden result or a published
conformance-count change. It routes revoked-Proxy errors; non-revocation
TypeErrors created by Proxy `[[Get]]` remain separate object-read work.

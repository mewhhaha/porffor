# Proxy `[[SetPrototypeOf]]` handler protocol

Status: implemented and focused structure-verified 2026-09-01 as a bounded T11
handler-acquisition and local-error-Realm change.

## Scope

This contract owns handler acquisition for Proxy `[[SetPrototypeOf]]` and the
Realm selection of the TypeErrors generated locally by that internal method:
reading the live Proxy slots, performing the full
`GetMethod(handler, "setPrototypeOf")`, routing an abrupt lookup, and either
calling the trap or forwarding to the target. The existing Boolean result,
non-extensible-target invariant, prototype comparison, ordinary fallback and
temporary-local release order remain unchanged.

Three public paths share `emit_object_set_prototype_of_i32`:

- `Object.setPrototypeOf`;
- `Reflect.setPrototypeOf`; and
- the legacy `Object.prototype.__proto__` setter required by ECMAScript.

The internal method is therefore observable beyond direct Reflect calls.

## Former acquisition defect

The SetPrototypeOf emitter previously loaded the handler payload from the
Proxy record, paired it with a fabricated `ValueKind::Object` tag and used an
ordinary-only property read. Function, Array and arguments handlers therefore
lost their representation before `GetMethod`, while Proxy handlers could skip
their own `[[Get]]` protocol. A lookup that completed abruptly also reached
`IsCallable` before the completion was routed.

The target payload and tag were read separately at the same site. That raw
layout knowledge duplicated the live-slot authority and made it possible for
the target and handler roles to drift independently.

## Typed acquisition boundary

The emitter now enters the shared typed live-slot reader once with one
`ProxySlotLocals` value containing distinct `ProxyTargetLocals` and
`ProxyHandlerLocals` roles.
`ProxyRevocationRoute::ObjectMutationRealmToActiveHandler` couples the existing
active-handler completion route to the trusted object-mutation Realm authority.
Revocation, a non-callable trap and the local non-extensible-target invariant
therefore allocate their TypeErrors from the called standard builtin's Realm.
The handler's retained payload and tag are both passed as the base and receiver
of the Proxy-aware property read.

Only `HEAP_OBJECT_BOXED_KIND_OFFSET` remains directly visible in this method,
and only to classify the current Object as a Proxy. The method may not read
the raw target payload/tag offsets, read the private handler-tag offset,
fabricate an Object handler tag, or call `emit_object_read_ordinary`.

## Observable order

For each Proxy reached by SetPrototypeOf, the emitted operation order is:

1. classify the current value as a Proxy;
2. reject revocation and retain the exact tagged target and handler;
3. perform Proxy-aware `GetMethod(handler, "setPrototypeOf")` with the exact
   handler as receiver;
4. route an abrupt lookup completion immediately, before `IsCallable`;
5. call a present callable trap through the shared Function-or-Proxy Call path
   with the exact handler as `this` and `(target, prototype)` as its two
   arguments; or
6. on an `undefined` or `null` trap, continue with the exact tagged target.

The existing post-call path still converts the trap result to Boolean. A
truthy trap result against a non-extensible target still requires SameValue
between the requested and current prototypes. A false trap result is still
returned to the public caller, which preserves its own specified behavior:
Object and the `__proto__` setter throw, while Reflect publishes the Boolean.
The three locally generated Proxy TypeErrors retain active-handler routing and
use the current standard builtin's trusted object-mutation Realm.

## Focused evidence

`wasm_proxy_set_prototype_of_handler_protocol.js` is a source-free Wasm-AOT
fixture covering:

- Function, Array, arguments and Proxy handlers;
- exact getter receiver, trap `this`, target and prototype arguments;
- a callable Proxy trap and its exact two-argument Call record;
- an abrupt Proxy-handler lookup sentinel;
- `null` and `undefined` fallback to a nested Proxy target; and
- created-Realm non-callable, invariant-mismatch and revoked errors through
  Object and Reflect.

`proxy_set_prototype_of_handler_protocol_structure` pins the typed live-slot
reader, full GetMethod path, immediate abrupt routing, exact Call record,
Realm-aware local error route, unchanged fallback/invariant/release structure,
all three public callers, the active CLI owner, fixture scenarios,
module-boundary census and this contract.
The module-boundary guard also rejects reintroduction of raw slot reads,
handler-tag fabrication or the ordinary-only handler read.

The write-phase marker `Verification pending` is retained here only as the
historical status superseded by the measured checkpoint below.

## Focused verification

The contract's focused command set is:

```sh
cargo fmt --all -- --check
git diff --check
cargo test -p lila-aot-wasm --test proxy_set_prototype_of_handler_protocol_structure -- --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_proxy_set_prototype_of_handler_protocol -- --exact --test-threads=1
./scripts/check-module-boundaries.sh
```

The structure target passes `4/4`, and the shared revocation-route structure
target also passes `4/4`. The exact CLI command in this block has no
individually attributed result here, nor does `cargo check -p lila-aot-wasm`;
T11 owns the collective seven-CLI and shared-gate results. No broad compile,
Test262 or published conformance result is claimed.

## Explicit nonclaims

This bounded change does not alter the SetPrototypeOf trap-result invariant,
ordinary object prototype mutation, immutable-prototype behavior, cycle
detection or public argument validation. Realm forwarding into nested Proxy
targets reached by the outlined `[[IsExtensible]]` and `[[GetPrototypeOf]]`
helpers remains separate debt, as do the `__proto__` setter's own wrapper-level
TypeErrors. This change does not retire a Test262 materializer, run a raw
Proxy/Reflect cohort, update published conformance counts, or close T11. Other
Proxy internal-method acquisition and recursive exotic-target work remain
separate T11 obligations.

# Proxy OwnPropertyKeys result ownership

Status: ownership and observable validation verified on Wasm AOT.

## Authority

The prospective Proxy `[[OwnPropertyKeys]]` trap and its call-result destination
are distinct, non-copyable `ProxyOwnKeysTrapLocals` and
`ProxyOwnKeysTrapResultLocals` roles. Previously both adjacent arguments were
`TaggedLocals`, so transposing them compiled: trap lookup and invocation could
write into the wrong scratch pair while post-trap validation read the other
pair.

Each of the three Object/Reflect producers (`Object.getOwnPropertyNames`,
`Object.getOwnPropertySymbols` and `Reflect.ownKeys`) now gives the result
authority to the single acquisition emitter, receives that same authority back,
and consumes it once in the corresponding post-trap validator. `Object.keys`,
`Object.values` and `Object.entries` share one EnumerableOwnProperties owner
that calls the `Reflect.ownKeys` builtin and so acquires no trap of its own.
Validators also accept the existing distinct `ProxyTargetLocals` instead of
adjacent raw payload/tag arguments. Trap, target, handler, and result roles
therefore cannot be transposed at these boundaries.

## Durable evidence

`proxy_own_keys_handler_protocol_structure` uses a Rust lexical identifier
census that excludes comments and ordinary, raw, byte, C-string, character,
and byte-character literals. It pins both exact role types, their lack of Copy
or Clone, the sole acquisition, all three producers, the returned ownership
transition, and exactly one typed validator consumption per producer.

On 2026-08-27, its seven focused structure tests passed, as did
`cargo check -p lila-aot-wasm --lib`. The exact `wasm_proxy_own_keys.js` and
`wasm_proxy_own_keys_handler_protocol.js` CLI witnesses each passed `1/1` on
the Wasm-AOT backend. Rustfmt's check mode and `git diff --check` also passed
for the scoped source, test, task, and contract files.

The initial ownership change altered no Proxy trap lookup, argument order, fallback,
revocation, call, validation, emitted instruction, public API, or conformance
count. That checkpoint did not claim that recursive Proxy descriptor validation or the full
Proxy/Reflect trees were complete.

## Observable validation

The 2026-09-27 follow-up retains the typed acquisition boundary and replaces
the validator's separate heap-layout scans with the canonical internal-method
owners. This follows [Proxy OwnPropertyKeys](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys):

- Read the returned object's length and elements through ordinary Get, preserving
  accessors, inherited indices and exact thrown values. Grow a private snapshot
  as elements arrive so a huge claimed length cannot preempt an early getter.
- Finish list conversion before rejecting duplicate keys.
- Observe target extensibility, own keys and every own descriptor in order.
  This includes nested Proxies, Arguments, Arrays, TypedArrays and namespaces.
- Check membership privately while accumulating descriptor observations, but
  delay invariant rejection until those observations finish. Later descriptor
  throws take precedence over an earlier missing key.
- Require every non-configurable key, and the exact key set for a
  non-extensible target.

The planner roots both canonical reflection operations for all three validator
callers. Namespace validation uses the same path, including export TDZ reads;
the separate namespace validator has been removed. Focused runtime regressions
and the existing namespace suite are the verification gates for this follow-up.

The final follow-up checkpoint passes 467 backend tests, seven ownership
structure checks, 32 focused runtime tests (including all 17 namespace tests),
and all 785 engine library tests, with no failures or ignored tests. The
affected pinned Test262 families pass 105/105, including Proxy `ownKeys` 54/54.
Both CLI protocol fixtures pass with `--host-surface test262`. Exact compiler
identity, commands, snapshots and the original 18/31 replay remain in the
[repair receipt](../required-fixes-20260926.json); semantic GC is still unfinished.

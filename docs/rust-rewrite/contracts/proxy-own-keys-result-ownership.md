# Proxy OwnPropertyKeys result ownership

Status: implemented as a source-equivalent Wasm-AOT compiler invariant.

## Authority

The prospective Proxy `[[OwnPropertyKeys]]` trap and its call-result destination
are distinct, non-copyable `ProxyOwnKeysTrapLocals` and
`ProxyOwnKeysTrapResultLocals` roles. Previously both adjacent arguments were
`TaggedLocals`, so transposing them compiled: trap lookup and invocation could
write into the wrong scratch pair while post-trap validation read the other
pair.

Each of the four Object/Reflect producers now gives the result authority to the
single acquisition emitter, receives that same authority back, and consumes it
once in the corresponding post-trap validator. Validators also accept the
existing distinct `ProxyTargetLocals` and, for `Object.keys`,
`ProxyHandlerLocals`, instead of adjacent raw payload/tag arguments. Trap,
target, handler, and result roles therefore cannot be transposed at these
boundaries.

## Durable evidence

`proxy_own_keys_handler_protocol_structure` uses a Rust lexical identifier
census that excludes comments and ordinary, raw, byte, C-string, character,
and byte-character literals. It pins both exact role types, their lack of Copy
or Clone, the sole acquisition, all four producers, the returned ownership
transition, and exactly one typed validator consumption per producer.

On 2026-08-27, its seven focused structure tests passed, as did
`cargo check -p lila-aot-wasm --lib`. The exact `wasm_proxy_own_keys.js` and
`wasm_proxy_own_keys_handler_protocol.js` CLI witnesses each passed `1/1` on
the Wasm-AOT backend. Rustfmt's check mode and `git diff --check` also passed
for the scoped source, test, task, and contract files.

This boundary changes no Proxy trap lookup, argument order, fallback,
revocation, call, validation, emitted instruction, public API, or conformance
count. It is not a claim that recursive Proxy descriptor validation or the full
Proxy/Reflect trees are complete.

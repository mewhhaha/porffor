# Throw-routing capability boundary

`ProxyCallThrowRouting` and `PrimitiveToNumberThrowRouting` are private policy
domains for two distinct completion boundaries. Each names exactly the two
currently valid owners: return the current function or leave the throw in the
completion tuple for an enclosing operation.

Neither domain implements cloning, copying, equality or debug formatting.
Their emitters borrow the policy wherever the generated control flow needs to
project it, and each projection uses an exhaustive match without a wildcard or
unreachable fallback. Adding a route is therefore a compile error until every
existing projection defines its behavior. Callers cannot clone or copy a
selected route within one raw emission, compare it to select an implicit
default, or expose it outside its owner module.

The proxy-aware call state machine needs to read one route at nine generated
throw exits, so its Boolean projection borrows the same policy throughout the
single Rust emission call. Primitive `ToNumber` similarly borrows one policy
at its BigInt and Symbol error sites. The named outer wrappers remain the only
route producers and the raw emitters remain private.

The focused structure guards pin both two-variant domains, their lack of
incidental capabilities, the borrowed exhaustive projections, the exact named
producers, and every current consumer.

This is a Rust authority change only. It does not alter emitted Wasm,
completion routing, error Realm selection, numeric conversion, proxy dispatch,
runtime-helper behavior, Test262 materialization or published conformance
counts.

```sh
cargo test -p lila-aot-wasm --test proxy_call_throw_routing_structure
cargo test -p lila-aot-wasm --test primitive_to_number_throw_routing_structure
```

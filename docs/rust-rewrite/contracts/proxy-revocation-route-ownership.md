# Proxy revocation route ownership

Status: implemented and focused structure-verified 2026-09-01 for the shared
live-Proxy slot reader's ten-producer route inventory.

## Boundary

`ProxyRevocationRoute::{CurrentFunctionRealm, ActiveHandler,
ObjectMutationRealmToActiveHandler, CurrentCompletion}` is the crate-private
one-shot authority that decides both how the shared live-Proxy slot reader
selects a revoked-Proxy TypeError Realm and how that completion leaves the
current body. It has ten exact producers and one consuming exhaustive router:

- Proxy `defineProperty`, `ownKeys`, `getOwnPropertyDescriptor` and direct
  `Reflect.set` use `CurrentFunctionRealm`;
- Proxy `getPrototypeOf`, `preventExtensions` and `isExtensible` use
  `ActiveHandler`;
- Proxy `setPrototypeOf` uses `ObjectMutationRealmToActiveHandler`; and
- Proxy `deleteProperty` and `HasProperty` use `CurrentCompletion`.

The authority derives no cloning, copying, formatting, equality, ordering,
hashing or default-construction capability. The router consumes it before any
live handler tag or target word is exposed. Reusing one route for a second
routing decision is therefore a move error, while adding a variant requires an
explicit throw policy in the exhaustive router before the crate builds.

## Durable evidence

`crates/lila-aot-wasm/tests/proxy_revocation_route_ownership_structure.rs`
Rust-lexically pins the crate-private attribute-free declaration, the recursive
eighteen-mention census, all ten producer mappings and the one complete
consuming router. Its fingerprint preserves the sentinel check, all four error
policies, return policies, closing `End`, and subsequent handler/target loads.
Direct method-route and UFCS censuses prevent an alternate caller from
bypassing the named producer inventory.

Focused verification commands:

```sh
cargo test -p lila-aot-wasm --test proxy_revocation_route_ownership_structure -- --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_proxy_define_property_handler_protocol -- --exact --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_proxy_get_prototype_of_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_proxy_set_prototype_of_handler_protocol -- --exact --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_proxy_delete_property_fixture -- --exact --test-threads=1
cargo test -p lila-aot-wasm --test proxy_reflect_set_handler_protocol_structure -- --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_proxy_reflect_set_handler_protocol -- --exact --test-threads=1
```

At the prior eight-producer checkpoint, the ownership target passed `4/4`.
The exact define-property current-Realm, get-prototype active-handler and
delete-property current-completion Wasm-AOT fixtures each passed `1/1`. At the
expanded checkpoint, the ten-producer structure target passes `4/4`; the exact
SetPrototypeOf Realm-aware active-handler and direct Reflect Set current-
function-Realm commands have no individually attributed current result here.
T11 owns the collective seven-CLI result. This does not claim a broad compile,
Test262 or published conformance result.

## Nonclaims

The SetPrototypeOf route is deliberately Realm-correcting: it retains the
existing active-handler completion route and message while selecting the
standard builtin's trusted object-mutation Realm instead of the main-Realm
runtime-error fallback. The accompanying handler-acquisition correction changes
SetPrototypeOf slot loads and trap lookup as specified by
`proxy-set-prototype-of-handler-protocol.md`; those changes are not attributed
to the route type itself. Adding direct `Reflect.set` remains source-equivalent:
it preserves its prior current-function-Realm route while its handler
acquisition changes under `proxy-reflect-set-handler-protocol.md`. Realm
forwarding into nested Proxy `[[IsExtensible]]` and `[[GetPrototypeOf]]` helper
calls remains separate debt. This contract changes no Proxy slot layout and
does not close the complete Proxy or Object task, recursive Proxy descriptor
protocols or broad Test262 coverage.

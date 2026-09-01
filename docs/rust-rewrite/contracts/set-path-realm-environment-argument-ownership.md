# Set-path Realm environment argument ownership

`SetPathRealmEnvironmentArgument` is the private two-row authority that emits
parameter 6 for the outlined object-mutation helpers. A trusted standard
builtin or set-path helper source emits the current environment; the global
fallback emits zero. The value represents exactly one helper ABI argument, so
it has no clone, copy, debug, comparison, default, conversion, or
representation capability and is consumed by one exhaustive match.

This is distinct from `ObjectMutationErrorRealm`. Direct mutation errors have
separate message and message-free sites that intentionally recompute their
Realm projection. It is also distinct from the already hardened object-read
Realm domains and from `ProxyRevocationRoute`.

The lexical structure guard pins the attribute-free two-row declaration, all
11 identifier mentions, the complete source projection, both exhaustive unit
observations, the sole product consumer, and the exact route census. The
consumer emits exactly one helper ABI argument: either
`LocalGet(current_env_local)` or `I64Const(0)`. Any second consuming observation
of the same authority now fails to compile, while an extra recomputation fails
the guarded route census.

This closure is source-equivalent. It changes no source classification, helper
signature, emitted instruction, stack order, Realm selection, error, or
completion behavior.

Focused verification:

```sh
cargo test -p lila-aot-wasm --test set_path_realm_environment_argument_ownership_structure -- --test-threads=1
cargo test -p lila-aot-wasm object_mutation_realm_projection_excludes_ordinary_lexical_environments -- --exact --test-threads=1
```

Runtime CLI verification is deferred, and Test262 remains deferred to the
shared checkpoint. This contract makes no broader Proxy, Reflect, object-write,
or conformance claim.

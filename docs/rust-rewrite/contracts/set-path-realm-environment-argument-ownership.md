# Set-path Realm environment argument ownership

Current dry source — 2026-10-05: the scalar set-path Realm argument owner
and its raw source-mirror target are retired in the atomic GC draft. Object
operations and native errors retain complete GC values and typed Realm owners.
The exact scalar argument, literal call counts and old module spelling below
belong to the historical implementation. Their earlier results do not verify
the current source batch.

The existing created-Realm Proxy Set CLI fixture and its registration witness
remain unchanged; current Engine Object/Reflect behavioral controls remain
intact. Compilation and runtime verification are deferred until all remaining
source families and representation/provider cleanup are finished.

`SetPathRealmEnvironmentArgument` is the private two-row authority that emits
parameter 6 for the outlined object-mutation helpers. A trusted standard
builtin or set-path helper source emits the current environment; the global
fallback emits the source Realm's function context in source bodies and zero
elsewhere. The value represents exactly one helper ABI argument, so
it has no clone, copy, debug, comparison, default, conversion, or
representation capability and is consumed by one exhaustive match.

This is distinct from `ObjectMutationErrorRealm`. Direct mutation errors have
separate message and message-free sites that intentionally recompute their
Realm projection. It is also distinct from the already hardened object-read
Realm domains and from `ProxyRevocationRoute`.

The lexical structure guard pins the attribute-free two-row declaration, all
11 identifier mentions, the complete source projection, both exhaustive unit
observations, the sole product consumer, and the exact route census. The
consumer emits exactly one helper ABI argument: `LocalGet(current_env_local)`,
the source Realm's function-context payload, or `I64Const(0)`. Any second consuming observation
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

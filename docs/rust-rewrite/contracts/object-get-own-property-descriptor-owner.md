# Object.getOwnPropertyDescriptor compiler owner

## Private ownership boundary

The complete `Object.getOwnPropertyDescriptor` compiler lives in the private
`builtins/object/get_own_property_descriptor.rs` module. Its 1,431-line compiler
family moved together: ordinary, Array, Arguments, TypedArray, Function and
Proxy descriptor materialization retain one implementation owner. The Object
parent contains only the private module declaration, and the standard
dispatcher retains one fixed builtin call.

The entry is visible only within `crate::builtins`. No raw helper, policy type
or representation-specific branch is exported. The recursive structure guard
pins private ownership, exact entry visibility, the single dispatcher call and
representative branches from every exotic-object path.

## Semantic boundary

This is a source-equivalent ownership move. The former 1,431-line selection has
SHA-256
`f656aa0168a19978df1e8698f87612426e073a8d37abd750a47cc970fba9ba24`.
After changing only the effective entry visibility and adding the inherent-impl
wrapper, the 1,435-line child has SHA-256
`1ef03c5ac9dddacea8f4979ac0c0128db2f93a4433576f977fcc2c9587918b89`;
the reduced 4,315-line parent has SHA-256
`f3d9ba2c52e218f5bf22764c389b723b3014dbed6b15cde1e821594c88e9df16`.
No emitted instruction, temporary-local order, descriptor algorithm or Realm
selection is intended to change. This lane claims no new descriptor behavior,
Test262 pass, shortcut retirement or published conformance change.

## Verification

```sh
cargo test -p lila-aot-wasm --test object_get_own_property_descriptor_owner_structure
cargo test -p lila-aot-wasm --test arguments_index_descriptor_structure
cargo test -p lila-aot-wasm --test array_index_descriptor_structure
cargo test -p lila-aot-wasm --test proxy_revocation_route_ownership_structure
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_object_descriptor_fixture -- --exact --test-threads=1
cargo fmt --all -- --check
git diff --check
```

Batch AP verification is green on 2026-08-28: the owner, Arguments neighbor,
Array neighbor and Proxy ownership structure targets pass `4/4`, `4/4`, `3/3`
and `4/4`; the exact object-descriptor CLI passes `1/1`; and `cargo xc` is
green.

## Proxy [[GetOwnProperty]] child

The Proxy step of the target loop is the private child
`builtins/object/get_own_property_descriptor/proxy.rs`, which implements 10.5.5
in step order and is the only place the trap is called. The parent declares
`mod proxy;` and calls `emit_proxy_get_own_property_descriptor` once per loop
iteration: a handler without the trap forwards to `[[ProxyTarget]]` and the loop
resolves that object in turn; otherwise the child leaves the answer in the
result locals.

- `targetDesc` (step 8) comes from a recursive call of this builtin on the
  target, so a Proxy target runs its own trap and the values that
  IsCompatiblePropertyDescriptor compares with SameValue are available.
- IsExtensible(target) (step 10) precedes ToPropertyDescriptor (step 11), whose
  HasProperty/Get reads of the trap result are observable.
- The converted descriptor is completed by 6.2.6.6
  (`emit_complete_property_descriptor`, which derives the side from `classify`),
  validated by 10.1.6.3 with `O` undefined plus the step 15 invariants, and
  published as a fresh object by the 6.2.6.4 owner in the executing builtin's
  Realm. The trap's own object is never the result.

This replaces the earlier behaviour that returned the trap result object itself
and validated only `configurable`/`writable` own data fields of it.
`crates/lila-aot-wasm/tests/object_get_own_property_descriptor_owner_structure.rs`
pins the split and `crates/lila-engine/tests/aot_proxy_get_own_property.rs` the
behaviour.

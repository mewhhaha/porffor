# Object.getOwnPropertyDescriptors compiler owner

## Private ownership boundary

The complete `Object.getOwnPropertyDescriptors` compiler lives in the private
`builtins/object/get_own_property_descriptors.rs` module. Its 182-line compiler
family moved together: `ToObject`, `Reflect.ownKeys`, per-key
`Reflect.getOwnPropertyDescriptor`, current-function-Realm result allocation,
property-key conversion and result definition retain one implementation owner.
The Object parent contains only the private module declaration, and the
standard dispatcher retains one fixed builtin call.

The entry is visible only within `crate::builtins`. No raw helper, policy type
or representation-specific branch is exported. The recursive structure guard
pins private ownership, exact entry visibility, the single dispatcher call and
representative markers from every observable phase.

## Semantic boundary

This is a source-equivalent ownership move. The former 182-line selection has
SHA-256
`34cdc9f6b4c1892c14e0175388c3c9d259a2f2e9fa0aa43439b19caa1d91e7c3`.
After changing only the effective entry visibility and adding the inherent-impl
wrapper, the 186-line child has SHA-256
`5529439ec3f7770a3a021a0ad2596fec56238908c2581337eeec08bf5fdafabd`.
Normalizing that visibility back to `pub(super)` reproduces the original hash
exactly. No emitted instruction, temporary-local order, observable call order,
Realm selection or descriptor result is intended to change. This lane claims
no new Object behavior, Test262 pass, shortcut retirement or published
conformance change.

## Verification

```sh
cargo test -p lila-aot-wasm --test object_get_own_property_descriptors_owner_structure
./target/debug/lila test262 run built-ins/Object/getOwnPropertyDescriptors/normal-object.js --threads 1 --execution-backend wasm --snapshot-dir <temporary-directory>
cargo fmt --all -- --check
git diff --check
```

At the 2026-08-28 Batch AQ checkpoint, `cargo xc` is green, the private-owner
structure target passes `4/4`, and the exact
`built-ins/Object/getOwnPropertyDescriptors/normal-object.js` leaf passes both
sloppy and strict Wasm-AOT executions (`2/2`) with every failure bucket at
zero. This source-equivalent move claims no new Object behavior, broader
Test262 result or published conformance-count change.

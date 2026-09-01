# Object.defineProperty descriptor roles

## Closed branch boundary

After `ToPropertyDescriptor` emits the 6.2.6.5 step 9 mixed-kind rejection,
the ordinary-object path carries exactly one private
`ObjectDefinePropertyDescriptorLocals::{Data, Accessor}` value. The `Data`
variant contains run-time-presence roles for `[[Value]]`, `[[Writable]]`,
`[[Enumerable]]` and `[[Configurable]]`. The `Accessor` variant contains roles
for `[[Get]]`, `[[Set]]`, `[[Enumerable]]` and `[[Configurable]]`.

One ownership-consuming exhaustive match constructs the corresponding
`PartialDescriptor<WasmLocals>`. The opposite kind's fields are structurally
absent, so the former contradictory combinations of an absent tagged carrier
and a separately present run-time flag cannot be expressed. The projection
uses `validate()` because each variant has only one descriptor side; the
`from_runtime_checked()` escape remains only at the distinct Arguments
`callee` boundary whose six fields are all run-time-possible.

Both `Object.defineProperty` producers name their branch variant and pass one
`ValidatedDescriptor<WasmLocals>` to the canonical ordinary-object algorithm.
The sixteen-argument `emit_object_define_entry` adapter and its
`presence_from_positional` translator are deleted.

## Semantic boundary

This is source-equivalent Rust type hardening. The existing emitted step 9
check still rejects mixed descriptors before either variant is built. Each
variant projects the same four run-time presences and the same opposite-side
absences as the retired adapter, so instruction order, temporary locals,
descriptor compatibility, property storage, error Realm and target identity
remain unchanged.

The exact object-descriptor CLI fixture covers data and accessor definition,
explicit `undefined`, mixed-kind rejection, generic updates, kind transitions
and frozen-property compatibility. This closure does not claim Proxy, Array,
TypedArray, Arguments-index or module-namespace descriptor completion.

## Verification

```sh
cargo test -p lila-aot-wasm --test object_define_property_descriptor_roles_structure
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_object_descriptor_fixture -- --exact --test-threads=1
cargo fmt --all -- --check
git diff --check
```

The recursive structure guard pins the exact two-case domain, its exhaustive
projection, both producers, direct validated-descriptor calls, retirement of
the positional adapter and the remaining single `from_runtime_checked()`
source obligation. At the 2026-08-28 Batch V checkpoint, the structure target
passed `4/4`, the exact object-descriptor CLI fixture passed `1/1`, and the
shared `cargo xc` gate was green. No semantic golden, broad descriptor suite or
Test262 baseline was rerun for this source-equivalent boundary closure.

Batch AO moves the complete descriptor carrier, Arguments-specialized helper
and builtin compiler family into the private
`builtins/object/define_property.rs` owner. The Object parent retains only
`mod define_property;`, and the standard dispatcher retains its one fixed
builtin call. The resulting 2,500-line child has SHA-256
`01ea9de92ace5f710bc6fcea6b7b4d64326e8d726e165920a06fdb1d8368b4c6`;
the reduced 5,751-line parent has SHA-256
`d8e910a3b8e2edcd7ab4e9fd6ee19507d86de6fbd6721d1da29655ffef817a53`.
This is a source-equivalent ownership move with no new descriptor behavior or
conformance claim. At the Batch AO checkpoint, `cargo xc` is green, the
strengthened owner target and five descriptor/proxy neighbors pass `25/25`, and
the exact object-descriptor CLI passes `1/1`.

# Accessor descriptor local roles

## Closed definition boundary

The three ordinary-object accessor definition entries consume one
`AccessorDescriptorLocals::{Getter, Setter, GetterAndSetter}` value. There is no
empty variant: a call named as an accessor definition must carry at least one
present accessor field.

Getter and setter values cross the boundary as distinct
`AccessorGetterLocals` and `AccessorSetterLocals` roles. Each role contains one
`TaggedLocals`, whose named payload and tag fields retain the existing
same-value boundary. The `GetterAndSetter` variant requires both distinct role
types, so transposing the two endpoints does not compile.

`emit_object_define_accessor`,
`emit_object_define_enumerable_accessor`, and
`emit_object_define_accessor_with_flag_local` all consume the closed domain.
One exhaustive match projects it into the descriptor lattice's `[[Get]]` and
`[[Set]]` presences before validation. Adding a fourth state therefore requires
stating both field presences, and neither a wildcard nor an independently
writable accessor-kind Boolean exists.

## Scope

This boundary covers object-literal accessors, public class accessors and host
prototype accessor installation. The separate `Object.defineProperty`
positional adapter was subsequently retired by the closed branch boundary in
`object-define-property-descriptor-roles.md`; that later change does not widen
this accessor-definition claim.

The migration changes Rust types and source spelling only. It does not change
emitted instruction order, descriptor attributes, function identity, host
publication, property ordering or Realm selection. Existing object-form, class
auto-accessor and TypedArray-accessor CLI fixtures are the focused behavior
controls.

## Verification

```sh
cargo test -p lila-aot-wasm --test accessor_descriptor_local_roles_structure
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_object_form_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli functions::run_wasm_class_auto_accessor_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli typed_array::run_wasm_backend_succeeds_for_typedarray_accessors_fixture -- --exact --test-threads=1
cargo fmt --all -- --check
git diff --check
```

The recursive source guard pins the exact nonempty role domain, the three typed
API signatures, one exhaustive presence projection, every role constructor and
all focused behavior/evidence links. Broad `Object.defineProperty`, Proxy,
Array-index and Arguments-index descriptor conformance remain outside this
source-equivalent type-hardening claim.

At the 2026-08-28 Batch U checkpoint, the structure target passed `4/4`, the
three exact CLI behavior controls passed `3/3`, and the shared `cargo xc` gate
was green. No semantic golden, broad descriptor suite or Test262 baseline was
rerun for this source-equivalent type hardening.

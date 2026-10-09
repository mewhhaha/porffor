# Accessor descriptor local roles

## Current GC definition boundary

The GC object owner consumes the nonempty
`AccessorDescriptorLocals::{Getter, Setter, GetterAndSetter}` domain at
`emit_object_define_accessor_with_flag_local`,
`emit_object_append_accessor_property_with_flags`, and
`emit_install_intrinsic_accessor_values`. An accessor definition cannot carry
neither endpoint. `AccessorGetterLocals` and `AccessorSetterLocals` borrow
complete rooted `ValueLocals`; exchanging their roles is a type error.

The private `objects/accessor_descriptor.rs` owner defines the generic
`AccessorDescriptor<T>` and distinct `AccessorGetter<T>`/`AccessorSetter<T>`
roles. Their concrete borrowed aliases are the GC publication boundary.
Intrinsic bootstrap uses the same domain over `StandardBuiltinId`, then
materializes the getter before the setter into owned rooted values. Borrowing
preserves both roles and the nonempty state, and release clears the setter
before the getter. The domain has no Clone, Copy, Default or optional endpoint
capabilities.

Only the object owner can project this definition into the descriptor lattice's
`[[Get]]` and `[[Set]]` presences. One exhaustive match names all three cases.
Fresh property append uses statically known presences, while object-literal
definition preserves absent endpoints through descriptor compatibility.

## Scope and evidence

This covers the named object-literal accessor boundary, intrinsic accessor
installation, and the Function/Arguments restricted accessors. General
`Object.defineProperty` and class descriptor construction continue through
their separate validated descriptor-lattice owners. The migration preserves
instruction and property order, flags, identities and Realm selection.

The source control pins the exact nonempty domain, all three borrowed
signatures, intrinsic materialization ordering, the sole field projection,
and the recursive producer/caller census. Existing object-form, class
auto-accessor and TypedArray-accessor CLI fixtures remain focused behavioral
controls. Current native and artifact verification is pending; the historical
checkpoint below does not establish that this GC revision passes.

```sh
cargo test -p lila-aot-wasm --test accessor_descriptor_local_roles_structure
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_supported_object_form_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli functions::run_wasm_class_auto_accessor_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli typed_array::run_wasm_backend_succeeds_for_typedarray_accessors_fixture -- --exact --test-threads=1
```

## Historical verification

At the 2026-08-28 Batch U checkpoint, the structure target passed `4/4`, the
three exact CLI behavior controls passed `3/3`, and the shared `cargo xc` gate
was green. No semantic golden, broad descriptor suite or Test262 baseline was
rerun for this source-equivalent type hardening.

const OBJECT_PARENT: &str = include_str!("../src/builtins/object.rs");
const OWNER: &str = include_str!("../src/builtins/object/get_own_property_descriptors.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/object-get-own-property-descriptors-owner.md"
);
const T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const T10: &str = include_str!("../../../tasks/10-object-model-descriptors-exotics.md");

#[test]
fn object_get_own_property_descriptors_has_one_private_module_owner() {
    assert_eq!(
        OBJECT_PARENT
            .matches("mod get_own_property_descriptors;")
            .count(),
        1
    );
    assert!(!OBJECT_PARENT.contains("pub mod get_own_property_descriptors;"));
    assert!(!OBJECT_PARENT.contains("compile_object_get_own_property_descriptors_builtin("));
    assert_eq!(
        OWNER
            .matches(
                "pub(in crate::builtins) fn compile_object_get_own_property_descriptors_builtin(",
            )
            .count(),
        1
    );
}

#[test]
fn fixed_dispatcher_entry_is_the_only_external_call() {
    assert_eq!(
        STANDARD
            .matches("ObjectGetOwnPropertyDescriptors =>")
            .count(),
        1
    );
    assert_eq!(
        STANDARD
            .matches("self.compile_object_get_own_property_descriptors_builtin(function)?")
            .count(),
        1
    );
    assert!(!STANDARD.contains("get_own_property_descriptors::"));
}

#[test]
fn complete_compiler_family_moved_together() {
    assert_eq!(OWNER.matches("impl<'a> FunctionBuilder<'a> {").count(), 1);
    assert_eq!(OWNER.matches("Result<(), EmitError>").count(), 1);
    assert_eq!(OWNER.matches("        Ok(())").count(), 1);
    for marker in [
        "StandardBuiltinId::ReflectOwnKeys.function_id()",
        "StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id()",
        "self.emit_value_to_current_function_realm_object_locals(",
        "self.emit_load_realm_intrinsic_prototype_or_global(",
        "self.emit_property_key_payload_from_value_local(",
        "self.emit_object_define_enumerable_data(",
    ] {
        assert!(OWNER.contains(marker), "missing compiler marker `{marker}`");
    }
    assert!(OWNER.contains("Object.getOwnPropertyDescriptors called on null or undefined"));
    assert!(!OBJECT_PARENT.contains("Object.getOwnPropertyDescriptors called on null or undefined"));
}

#[test]
fn owner_evidence_records_scope_and_nonclaim() {
    for evidence in [CONTRACT, T02, T10] {
        assert!(evidence.contains("object/get_own_property_descriptors.rs"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new Object behavior"));
    }
}

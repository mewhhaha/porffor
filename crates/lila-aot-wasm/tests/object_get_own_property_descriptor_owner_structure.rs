const OBJECT_PARENT: &str = include_str!("../src/builtins/object.rs");
const OWNER: &str = include_str!("../src/builtins/object/get_own_property_descriptor.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/object-get-own-property-descriptor-owner.md"
);
const T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const T10: &str = include_str!("../../../tasks/10-object-model-descriptors-exotics.md");

#[test]
fn object_get_own_property_descriptor_has_one_private_module_owner() {
    assert_eq!(
        OBJECT_PARENT
            .matches("mod get_own_property_descriptor;")
            .count(),
        1
    );
    assert!(!OBJECT_PARENT.contains("pub mod get_own_property_descriptor;"));
    assert!(!OBJECT_PARENT.contains("compile_object_get_own_property_descriptor_builtin("));
    assert_eq!(
        OWNER
            .matches(
                "pub(in crate::builtins) fn compile_object_get_own_property_descriptor_builtin(",
            )
            .count(),
        1
    );
}

#[test]
fn fixed_dispatcher_entry_is_the_only_external_call() {
    assert_eq!(
        STANDARD
            .matches("ObjectGetOwnPropertyDescriptor =>")
            .count(),
        1
    );
    assert_eq!(
        STANDARD
            .matches("self.compile_object_get_own_property_descriptor_builtin(function)?")
            .count(),
        1
    );
    assert!(!STANDARD.contains("get_own_property_descriptor::"));
}

#[test]
fn complete_compiler_family_moved_together() {
    assert_eq!(OWNER.matches("impl<'a> FunctionBuilder<'a> {").count(), 1);
    assert_eq!(OWNER.matches("Result<(), EmitError>").count(), 1);
    assert_eq!(OWNER.matches("        Ok(())").count(), 1);
    for marker in [
        "self.emit_load_live_proxy_slots(",
        "self.emit_array_descriptor_kind_for_index(",
        "self.emit_arguments_parameter_map_read(",
        "self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(",
        "self.emit_alloc_data_descriptor_from_locals_with_flag_locals(",
    ] {
        assert!(OWNER.contains(marker), "missing compiler marker `{marker}`");
    }
    for owner_only_marker in [
        "self.emit_load_live_proxy_slots(",
        "self.emit_arguments_parameter_map_read(",
        "self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(",
        "self.emit_alloc_data_descriptor_from_locals_with_flag_locals(",
    ] {
        assert!(!OBJECT_PARENT.contains(owner_only_marker));
    }
}

#[test]
fn owner_evidence_records_scope_and_nonclaim() {
    for evidence in [CONTRACT, T02, T10] {
        assert!(evidence.contains("object/get_own_property_descriptor.rs"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new descriptor behavior"));
    }
}

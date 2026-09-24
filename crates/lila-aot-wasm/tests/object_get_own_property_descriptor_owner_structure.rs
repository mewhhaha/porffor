const OBJECT_PARENT: &str = include_str!("../src/builtins/object.rs");
const OWNER: &str = include_str!("../src/builtins/object/get_own_property_descriptor.rs");
const PROXY_CHILD: &str =
    include_str!("../src/builtins/object/get_own_property_descriptor/proxy.rs");
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
        "self.emit_proxy_get_own_property_descriptor(",
        "self.emit_array_descriptor_kind_for_index(",
        "self.emit_arguments_parameter_map_read(",
        "self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(",
        "self.emit_alloc_data_descriptor_from_locals_with_flag_locals(",
    ] {
        assert!(OWNER.contains(marker), "missing compiler marker `{marker}`");
    }
    assert!(PROXY_CHILD.contains("self.emit_load_live_proxy_slots("));
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

#[test]
fn proxy_get_own_property_is_one_private_child_step() {
    // The parent declares the child privately and calls its single entry from
    // the target loop; nothing of 10.5.5 stays in the parent.
    assert_eq!(OWNER.matches("\nmod proxy;\n").count(), 1);
    assert!(!OWNER.contains("pub mod proxy;"));
    assert!(!OWNER.contains("pub(crate) mod proxy;"));
    assert_eq!(
        OWNER
            .matches("self.emit_proxy_get_own_property_descriptor(")
            .count(),
        1
    );
    for proxy_only in [
        "self.emit_load_live_proxy_slots(",
        "emit_function_or_proxy_call",
        "\"getOwnPropertyDescriptor\"",
        "getOwnPropertyDescriptor trap",
        "emit_direct_own_descriptor_fact(",
        "emit_to_property_descriptor(",
    ] {
        assert!(
            !OWNER.contains(proxy_only),
            "parent regained `{proxy_only}`"
        );
    }
    // A handled step exits the loop; a trap-less handler loops on the target.
    let parent: String = OWNER.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(parent.contains(concat!(
        "handled:proxy_handled_local,",
        "result:TaggedLocals::new(self.result_local,self.result_tag_local),},function,)?;",
        "function.instruction(&Instruction::LocalGet(proxy_handled_local));",
        "function.instruction(&Instruction::I64Eqz);",
        "function.instruction(&Instruction::BrIf(0));",
    )));

    assert_eq!(
        PROXY_CHILD
            .matches("pub(super) fn emit_proxy_get_own_property_descriptor(")
            .count(),
        1
    );
    assert_eq!(PROXY_CHILD.matches("pub(super) fn ").count(), 1);
    assert_eq!(PROXY_CHILD.matches("pub(crate)").count(), 0);
    for step in [
        "// Steps 1-3.",
        "// Step 4: GetMethod(handler, \"getOwnPropertyDescriptor\").",
        "// Step 5: return ? target.[[GetOwnProperty]](P).",
        "// Step 6.",
        "// Step 7: an Object of any representation, or undefined.",
        "// Step 8.",
        "// Step 9.",
        "// Step 10 precedes the observable conversion of step 11.",
        "// Step 12.",
        "// Steps 13-14.",
        "// Step 15.",
        "// Step 16,",
    ] {
        assert_eq!(PROXY_CHILD.matches(step).count(), 1, "step `{step}`");
    }
    // targetDesc comes from the recursive builtin, never from the value-free
    // fact, so a Proxy target runs its own trap and SameValue sees values.
    assert!(PROXY_CHILD
        .contains(".get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())"));
    assert_eq!(PROXY_CHILD.matches("self.emit_direct_js_call(").count(), 1);
    assert!(!PROXY_CHILD.contains("emit_direct_own_descriptor_fact("));
    // IsCompatiblePropertyDescriptor compares values with SameValue for the
    // accessor pair and the frozen data value.
    let compatibility = PROXY_CHILD
        .split_once("    fn emit_proxy_get_own_property_compatibility(")
        .expect("compatibility owner")
        .1
        .split_once("    fn emit_proxy_get_own_property_type_error_if(")
        .expect("compatibility end")
        .0;
    assert_eq!(
        compatibility
            .matches("self.emit_tagged_payload_same_value_i32(")
            .count(),
        2
    );
    for check in [
        "completed.emit_configurable_i32(function);",
        "completed.emit_enumerable_i32(function);",
        "completed.emit_accessor_i32(function);",
        "completed.emit_writable_i32(function);",
        "(completed.getter(), target_descriptor.get),",
        "(completed.setter(), target_descriptor.set),",
        "let requested = completed.value();",
    ] {
        assert!(compatibility.contains(check), "missing `{check}`");
    }
    assert!(!PROXY_CHILD.contains("_ =>"));

    assert!(CONTRACT.contains("get_own_property_descriptor/proxy.rs"));
    assert!(CONTRACT.contains("10.5.5"));
}

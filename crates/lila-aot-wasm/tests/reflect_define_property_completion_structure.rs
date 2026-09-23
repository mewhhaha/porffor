const REFLECT: &str = include_str!("../src/builtins/reflect.rs");
const OBJECT: &str = include_str!("../src/builtins/object/define_property.rs");
const BINARY_DATA: &str = include_str!("../src/builtins/binary_data.rs");
const TYPED_ARRAY_DEFINE: &str = include_str!("../src/builtins/binary_data/define_property.rs");
const OBJECTS: &str = include_str!("../src/objects.rs");
const DESCRIPTOR: &str = include_str!("../../lila-ir/src/property_descriptor.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing {end}"))
        .0
}

fn normalized(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

fn ordered(source: &str, witnesses: &[&str]) {
    let mut remaining = source;
    for witness in witnesses {
        remaining = remaining
            .split_once(witness)
            .unwrap_or_else(|| panic!("missing ordered witness {witness}"))
            .1;
    }
}

fn reflect_define() -> &'static str {
    bounded(
        REFLECT,
        "pub(crate) fn compile_reflect_define_property_builtin(",
        "pub(crate) fn compile_reflect_delete_property_builtin(",
    )
}

fn object_define() -> &'static str {
    OBJECT
        .split_once("pub(in crate::builtins) fn compile_object_define_property_builtin(")
        .expect("Object definition owner")
        .1
}

#[test]
fn private_forwarding_drops_the_prototype_only_after_a_trap_did_not_receive_the_object() {
    let source = normalized(reflect_define());
    ordered(
        &source,
        &[
            "self.emit_alloc_reflect_descriptor_object(",
            "self.emit_proxy_define_property_trap_result(",
            "self.emit_proxy_define_property_trap_invariants(",
            "function.instruction(&Instruction::LocalGet(handled_local));function.instruction(&Instruction::I64Eqz);function.instruction(&Instruction::If(BlockType::Empty));",
            "self.store_i64_const_at_offset(descriptor_payload_local,HEAP_PROTOTYPE_OFFSET,0,function,);",
            "self.store_i64_const_at_offset(descriptor_payload_local,HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,ValueKind::Null.tag()asu64,function,);",
            "self.emit_function_handle_call(reflect_define_payload_local,",
        ],
    );
    assert_eq!(source.matches("HEAP_PROTOTYPE_OFFSET").count(), 1);
    assert_eq!(
        source.matches("HEAP_OBJECT_PROTOTYPE_TAG_OFFSET").count(),
        1
    );

    let trap = bounded(
        OBJECTS,
        "pub(crate) fn emit_proxy_define_property_trap_result(",
        "pub(crate) fn emit_object_boxed_kind_for_tag(",
    );
    let trap = normalized(trap);
    ordered(
        &trap,
        &[
            "self.emit_is_callable_i32(",
            "self.emit_function_or_proxy_call_with_throw_propagation(",
            "function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::LocalSet(handled_local));",
            "function.instruction(&Instruction::Else);",
        ],
    );
}

#[test]
fn object_private_forwarding_uses_the_same_nonescape_boundary() {
    let source = normalized(object_define());
    ordered(
        &source,
        &[
            "self.emit_from_present_property_descriptor(",
            "self.emit_proxy_define_property_trap_result(",
            "self.emit_proxy_define_property_trap_invariants(",
            "function.instruction(&Instruction::LocalGet(proxy_handled_local));function.instruction(&Instruction::I64Eqz);function.instruction(&Instruction::If(BlockType::Empty));",
            "self.store_i64_const_at_offset(descriptor_payload_local,HEAP_PROTOTYPE_OFFSET,0,function,);",
            "self.store_i64_const_at_offset(descriptor_payload_local,HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,ValueKind::Null.tag()asu64,function,);",
            "self.emit_function_value_payload(&object_define_meta,function)?;",
            "self.emit_function_handle_call(",
        ],
    );
    assert_eq!(source.matches("HEAP_PROTOTYPE_OFFSET").count(), 1);
    assert_eq!(
        source.matches("HEAP_OBJECT_PROTOTYPE_TAG_OFFSET").count(),
        1
    );
}

#[test]
fn both_public_methods_dispatch_numeric_indexes_through_the_validated_boolean_operation() {
    assert_eq!(BINARY_DATA.matches("mod define_property;").count(), 1);
    assert!(!BINARY_DATA.contains("pub mod define_property;"));
    for source in [reflect_define(), object_define()] {
        assert_eq!(
            source
                .matches("emit_typed_array_define_index_property(")
                .count(),
            1
        );
        assert!(!source.contains("emit_typed_array_element_write_from_locals("));
        ordered(
            source,
            &[
                "let definition_descriptor = ",
                ".from_runtime_checked();",
                "emit_canonical_numeric_index_string(",
                "emit_typed_array_define_index_property(",
                "&definition_descriptor,",
                "self.emit_return_current_completion_if_throw(function);",
            ],
        );
    }
    ordered(
        reflect_define(),
        &[
            "emit_typed_array_define_index_property(",
            "self.emit_return_current_completion_if_throw(function);",
            "self.emit_return_current_completion(function);",
            "self.emit_ordinary_is_extensible_i32(",
            "emit_function_handle_call_without_throw_propagation(",
        ],
    );
    ordered(
        object_define(),
        &[
            "emit_typed_array_define_index_property(",
            "self.emit_return_current_completion_if_throw(function);",
            "Instruction::LocalGet(typed_array_define_success_local)",
            "Instruction::I64Eqz",
            "Cannot define incompatible TypedArray index descriptor",
        ],
    );
    for owner in [
        "FunctionBuilder::compile_object_define_property_builtin",
        "FunctionBuilder::compile_reflect_define_property_builtin",
    ] {
        assert!(
            DESCRIPTOR.contains(owner),
            "missing runtime validation obligation {owner}"
        );
    }
}

#[test]
fn index_definition_preserves_rejection_and_coercion_as_distinct_completion_paths() {
    assert!(TYPED_ARRAY_DEFINE.contains("descriptor: &WasmDescriptor,"));
    assert!(!TYPED_ARRAY_DEFINE.contains("descriptor: &WasmPartialDescriptor,"));
    for forbidden in [
        "set_completion_kind(",
        "emit_throw_",
        "without_throw_propagation",
        "HEAP_TYPED_ARRAY_",
        "HEAP_ARRAY_BUFFER_",
    ] {
        assert!(
            !TYPED_ARRAY_DEFINE.contains(forbidden),
            "operation bypasses its completion/witness contract: {forbidden}"
        );
    }
    ordered(
        TYPED_ARRAY_DEFINE,
        &[
            "Instruction::I64Const(0)",
            "Instruction::LocalSet(result_local)",
            "emit_typed_array_valid_integer_index_i32(",
            "Instruction::BrIf(0)",
            "&descriptor.configurable",
            "&descriptor.enumerable",
            "&descriptor.writable",
            "&descriptor.get",
            "&descriptor.set",
            "descriptor.value.value()",
            "emit_typed_array_element_write_from_locals(",
            "self.emit_return_current_completion_if_throw(function);",
            "Instruction::I64Const(1)",
            "Instruction::LocalSet(result_local)",
        ],
    );
    let reservations: Vec<_> = TYPED_ARRAY_DEFINE
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("let ")?
                .strip_suffix(" = self.reserve_temp_local();")
        })
        .collect();
    let releases: Vec<_> = TYPED_ARRAY_DEFINE
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("self.release_temp_local(")?
                .strip_suffix(");")
        })
        .collect();
    assert_eq!(reservations.into_iter().rev().collect::<Vec<_>>(), releases);
}

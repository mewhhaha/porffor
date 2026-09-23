const FUNCTIONS: &str = include_str!("../src/functions.rs");
const OWNER: &str = include_str!("../src/functions/throw_type_error.rs");
const HOST: &str = include_str!("../src/builtins/host.rs");
const BOOTSTRAP: &str = include_str!("../src/builtins/bootstrap.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");
const HEAP: &str = include_str!("../src/heap.rs");
const MODULE: &str = include_str!("../src/module.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .expect(start)
        .1
        .split_once(end)
        .expect(end)
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn realm_bootstrap_publishes_the_thrower_before_exposing_the_function_family() {
    assert_eq!(FUNCTIONS.matches("\nmod throw_type_error;\n").count(), 1);
    let bootstrap = bounded(
        HOST,
        "pub(crate) fn compile_host_create_realm_builtin(",
        "pub(crate) fn compile_host_agent_start_builtin(",
    );
    let function_prototype = bootstrap
        .find("self.emit_store_realm_function_prototype(&realm_functions, function);")
        .unwrap();
    let thrower = bootstrap
        .find("self.emit_initialize_realm_throw_type_error(&realm_functions, function)?;")
        .unwrap();
    let constructor = bootstrap
        .find("self.emit_bind_realm_function_constructor_prototype(")
        .unwrap();
    assert!(function_prototype < thrower && thrower < constructor);
    assert_eq!(
        bootstrap
            .matches("self.emit_initialize_realm_throw_type_error(")
            .count(),
        1
    );
    let entry = bounded(
        BOOTSTRAP,
        "pub(crate) fn init_throw_type_error_intrinsic(",
        "pub(crate) fn init_reflect_object(",
    );
    assert!(entry.contains("StandardBuiltinId::ThrowTypeError.function_id()"));
    assert!(entry.contains("NonArrayRealmIntrinsicSlot::ThrowTypeError"));
}

#[test]
fn created_realm_thrower_uses_canonical_function_allocation_and_one_owned_identity() {
    let owner = bounded(
        OWNER,
        "pub(crate) fn emit_initialize_realm_throw_type_error(",
        "pub(crate) fn emit_load_required_function_realm_throw_type_error(",
    );
    let owner = normalized(owner);
    assert!(owner.contains("context:&RealmFunctionMaterializationContext"));
    assert!(owner.contains("self.emit_function_value_payload_in_realm(&thrower_meta,context,thrower_payload_local,function,)?;"));
    assert!(owner.contains("self.emit_store_non_array_realm_intrinsic(context.realm.index(),NonArrayRealmIntrinsicSlot::ThrowTypeError,thrower_payload_local,function,);"));
    assert!(owner.contains("fornamein[\"caller\",\"arguments\"]"));
    assert!(owner.contains("self.emit_object_append_accessor_property_with_flags(context.function_prototype_local,key_local,Some((thrower_payload_local,thrower_tag_local)),Some((thrower_payload_local,thrower_tag_local)),false,true,function,)?;"));
    assert!(!owner.contains("GlobalSet"));
    assert!(!owner.contains("emit_alloc_plain_object"));
}

#[test]
fn invocation_carries_the_called_intrinsics_realm_and_allocates_a_fresh_error() {
    let allocation = bounded(
        FUNCTIONS,
        "pub(crate) fn emit_function_value_payload_with_prototype_materialization(",
        "pub(crate) fn emit_function_value_payload_in_realm(",
    );
    let allocation = normalized(allocation);
    assert!(allocation
        .contains("Some(StandardBuiltinId::EvalFunction|StandardBuiltinId::ThrowTypeError|StandardBuiltinId::StringConstructor)"));
    assert!(allocation.contains("self.store_i64_local_at_offset(object_local,HEAP_FUNCTION_ENV_HANDLE_OFFSET,object_local,function,);"));
    let body = normalized(bounded(
        STANDARD,
        "StandardBuiltinId::ThrowTypeError => {",
        "StandardBuiltinId::TypedArrayConstructor => {",
    ));
    assert!(body.contains("self.emit_throw_current_function_realm_type_error_without_message(self.result_local,self.result_tag_local,function,)?;"));
    assert!(!body.contains("emit_throw_runtime_error("));
    assert!(!body.contains("GlobalGet"));
}

#[test]
fn arguments_publish_both_accessors_only_after_required_realm_intrinsic_loading() {
    let loader = normalized(
        OWNER
            .split_once("pub(crate) fn emit_load_required_function_realm_throw_type_error(")
            .unwrap()
            .1,
    );
    for field in [
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "HEAP_REALM_INTRINSICS_THROW_TYPE_ERROR_OFFSET",
    ] {
        assert!(loader.contains(field));
    }
    assert!(loader.contains("Instruction::LocalGet(base_local)"));
    assert!(loader.contains("Instruction::LocalGet(result_local)"));
    assert_eq!(loader.matches("Instruction::Unreachable").count(), 2);
    assert!(!loader.contains("GlobalGet"));
    let arguments = bounded(
        FUNCTIONS,
        "pub(crate) fn emit_arguments_object_payload(",
        "pub(crate) fn emit_arguments_length(",
    );
    let arguments = normalized(arguments);
    let load = arguments.find("self.emit_load_required_function_realm_throw_type_error(iterator_payload_local,thrower_payload_local,function,);").unwrap();
    let getter = arguments
        .find("HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,thrower_payload_local,")
        .unwrap();
    let setter = arguments
        .find("HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET,thrower_payload_local,")
        .unwrap();
    assert!(load < getter && getter < setter);
    assert!(arguments.contains("self.release_temp_local(thrower_payload_local);"));
    assert!(!arguments.contains("THROW_TYPE_ERROR_GLOBAL_INDEX"));
}

#[test]
fn existing_realm_slot_and_entry_global_remain_traced_pointer_roots() {
    let slot = bounded(HEAP, "name: \"%ThrowTypeError%\",", "    },");
    assert!(slot.contains("offset: HEAP_REALM_INTRINSICS_THROW_TYPE_ERROR_OFFSET"));
    assert!(slot.contains("pointer: true"));
    let root = bounded(MODULE, "name: \"%ThrowTypeError%\",", "    },");
    assert!(root.contains("index: THROW_TYPE_ERROR_GLOBAL_INDEX"));
}

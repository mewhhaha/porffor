const INSTALLER_SOURCE: &str = include_str!("../src/intrinsics/function.rs");
const FUNCTION_BODY_SOURCE: &str = include_str!("../src/builtins/function.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const OBJECT_BUILTIN_SOURCE: &str = include_str!("../src/builtins/object.rs");
const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const CATALOG_SOURCE: &str = include_str!("../../lila-ir/src/builtins/catalog.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_function_prototype_symbol_has_instance.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/function-prototype-symbol-has-instance.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier = source.find(earlier).expect("earlier operation");
    let later = source.find(later).expect("later operation");
    assert!(earlier < later, "{earlier} must precede {later}");
}

#[test]
fn intrinsic_is_one_rooted_nonconstructable_catalog_identity() {
    let catalog = bounded(
        CATALOG_SOURCE,
        "    FunctionPrototypeSymbolHasInstance {",
        "\n}\n\nimpl StandardBuiltinId {",
    );
    assert!(catalog.contains("=> BUILTIN_FUNCTION_PROTOTYPE_SYMBOL_HAS_INSTANCE_FUNCTION_ID"));
    assert!(catalog.contains("debug: \"Function.prototype[Symbol.hasInstance]\""));
    assert!(catalog.contains("flags: []"));
    assert!(catalog.contains("installer: None"));
    assert!(catalog.contains("native: \"[Symbol.hasInstance]\""));

    let roots = bounded(
        PLANNING_SOURCE,
        "        if builtin == StandardBuiltinId::FunctionConstructor {",
        "        if builtin == StandardBuiltinId::DisposableStackConstructor {",
    );
    assert_eq!(
        roots
            .matches("StandardBuiltinId::FunctionPrototypeSymbolHasInstance,")
            .count(),
        1
    );
    assert!(roots.contains("self.require_standard_builtin(dependency)"));

    let length = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn standard_builtin_length(builtin: StandardBuiltinId) -> u64 {",
        "pub(crate) fn host_builtin_length(builtin: HostBuiltinId) -> u64 {",
    );
    assert!(length.contains("StandardBuiltinId::FunctionPrototypeSymbolHasInstance"));
    assert!(length.contains("=> 1,"));

    let dispatch = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::FunctionPrototypeSymbolHasInstance => {",
        "            StandardBuiltinId::FunctionPrototypeCall => {",
    );
    assert_eq!(
        dispatch
            .matches("FunctionBuiltin::PrototypeSymbolHasInstance")
            .count(),
        1
    );

    let body = bounded(
        FUNCTION_BODY_SOURCE,
        "            FunctionBuiltin::PrototypeSymbolHasInstance => {",
        "            FunctionBuiltin::PrototypeCall => {",
    );
    assert_eq!(body.matches("self.emit_builtin_arg_to_locals(").count(), 1);
    assert_eq!(
        body.matches("self.emit_ordinary_has_instance_from_locals(")
            .count(),
        1
    );
    assert!(body.contains("ValueKind::Boolean.tag()"));
    assert!(!body.contains("emit_instanceof_operator_from_locals"));
}

#[test]
fn entry_realm_installs_the_symbol_property_with_all_false_attributes() {
    let install = bounded(
        INSTALLER_SOURCE,
        "        let has_instance_meta = self",
        "        self.release_temp_local(prototype_object_local);",
    );
    assert_eq!(
        install
            .matches("StandardBuiltinId::FunctionPrototypeSymbolHasInstance.function_id()")
            .count(),
        1
    );
    assert!(install.contains("lila_ir::WellKnownSymbol::HasInstance.description()"));
    assert!(install.contains("property_key_symbol_payload"));
    assert!(install.contains("self.emit_function_value_payload(&has_instance_meta, function)?"));
    assert!(install.contains("ValueKind::Function.tag()"));
    assert!(install.contains(
        "prototype_object_local,\n            key_local,\n            payload_local,\n            tag_local,\n            false,\n            false,\n            false,"
    ));
    assert_before(
        install,
        "property_key_symbol_payload",
        "self.emit_object_append_data_property_with_flags(",
    );
}

#[test]
fn created_realm_installs_a_fresh_realm_local_function_through_the_typed_context() {
    let meta = bounded(
        HOST_SOURCE,
        "        let function_prototype_has_instance_meta = self",
        "        let object_meta = self",
    );
    assert_eq!(
        meta.matches("StandardBuiltinId::FunctionPrototypeSymbolHasInstance.function_id()")
            .count(),
        1
    );

    let install = bounded(
        HOST_SOURCE,
        "        let has_instance_payload_local = self.reserve_temp_local();",
        "        for (_, prototype_local) in &typed_array_prototype_locals {",
    );
    for operation in [
        "self.emit_function_value_payload_in_realm(",
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "lila_ir::WellKnownSymbol::HasInstance",
        "self.emit_define_realm_function_prototype_symbol_data_with_flags(",
    ] {
        assert!(
            install.contains(operation),
            "missing created-realm step: {operation}"
        );
    }
    assert!(install.contains(
        "has_instance_payload_local,\n            tag_local,\n            false,\n            false,\n            false,"
    ));
    assert_before(
        install,
        "self.emit_function_value_payload_in_realm(",
        "self.emit_define_realm_function_prototype_symbol_data_with_flags(",
    );
    assert_before(
        install,
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "self.emit_define_realm_function_prototype_symbol_data_with_flags(",
    );

    let helper = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_define_realm_function_prototype_symbol_data_with_flags(",
        "    pub(crate) fn emit_bind_realm_function_constructor_prototype(",
    );
    assert!(helper.contains("context: &RealmFunctionMaterializationContext"));
    assert!(helper.contains("symbol: lila_ir::WellKnownSymbol"));
    assert!(helper.contains("property_key_symbol_payload(symbol.description())"));
    assert!(helper.contains("context.function_prototype_local"));
    assert!(!helper.contains("FUNCTION_PROTOTYPE_GLOBAL_INDEX"));
}

#[test]
fn has_instance_dispatch_is_a_closed_noncopyable_two_entry_domain() {
    let request = bounded(
        OPERATIONS_SOURCE,
        "enum HasInstanceRequestLocals {",
        "#[derive(Clone, Copy)]\n#[repr(u64)]\nenum HasInstanceRuntimeState",
    );
    assert!(request.contains("InstanceofOperator"));
    assert!(request.contains("object: HasInstanceValueLocals"));
    assert!(request.contains("constructor: HasInstanceValueLocals"));
    assert!(request.contains("OrdinaryHasInstance"));
    assert!(!request.contains("bool"));
    assert!(!request.contains("_ =>"));
    assert!(OPERATIONS_SOURCE.contains("#[must_use]\nenum HasInstanceRequestLocals"));
    assert!(!OPERATIONS_SOURCE.contains("impl Copy for HasInstanceRequestLocals"));

    let operator_wrapper = bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn emit_instanceof_operator_from_locals(",
        "    pub(crate) fn emit_ordinary_has_instance_from_locals(",
    );
    assert!(operator_wrapper.contains("HasInstanceRequestLocals::InstanceofOperator"));
    assert!(operator_wrapper.contains("object: HasInstanceValueLocals::new("));
    assert!(operator_wrapper.contains("constructor: HasInstanceValueLocals::new("));

    let ordinary_wrapper = bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn emit_ordinary_has_instance_from_locals(",
        "    fn emit_has_instance_request(",
    );
    assert!(ordinary_wrapper.contains("HasInstanceRequestLocals::OrdinaryHasInstance"));
    assert!(ordinary_wrapper.contains("constructor: HasInstanceValueLocals::new("));
    assert!(ordinary_wrapper.contains("object: HasInstanceValueLocals::new("));

    let emitter = bounded(
        OPERATIONS_SOURCE,
        "    fn emit_has_instance_request(",
        "    pub(crate) fn emit_update_delta(",
    );
    assert!(emitter.contains("let (state, constructor, object) = match request"));
    assert!(!emitter.contains("_ =>"));
    assert!(emitter.contains("property_key_symbol_payload(\"Symbol.hasInstance\")"));
    assert!(emitter.contains("self.emit_indirect_call_from_locals("));
    assert!(emitter.contains("self.emit_to_boolean_payload_from_tagged_locals("));
    assert!(emitter.contains("FUNCTION_FLAG_BOUND"));
    assert!(emitter.contains("HasInstanceRuntimeState::InstanceofOperator as i64"));
    assert!(emitter.contains("self.strings.payload(\"prototype\")"));
    assert!(emitter.contains("self.emit_object_read("));
    assert!(emitter.contains("self.emit_object_get_prototype_of("));

    let absent_handler = bounded(
        emitter,
        "function.instruction(&Instruction::I32Or);",
        "self.emit_is_callable_i32(handler_tag_local, handler_payload_local, function)?;",
    );
    assert!(absent_handler
        .contains("self.emit_is_callable_i32(constructor_tag_local, constructor_payload_local"));
    assert!(absent_handler.contains("Right-hand side of 'instanceof' is not callable"));
    assert_before(
        absent_handler,
        "self.emit_is_callable_i32(constructor_tag_local, constructor_payload_local",
        "HasInstanceRuntimeState::OrdinaryHasInstance as i64",
    );
    assert_before(
        emitter,
        "FUNCTION_FLAG_BOUND",
        "self.strings.payload(\"prototype\")",
    );

    let define_property = bounded(
        OBJECT_BUILTIN_SOURCE,
        "    pub(super) fn compile_object_define_property_builtin(",
        "    pub(super) fn compile_object_get_own_property_descriptor_builtin(",
    );
    assert!(define_property.contains("self.emit_object_define_entry("));
    assert!(!define_property.contains("FUNCTION_FLAG_BOUND | FUNCTION_FLAG_IS_HTMLDDA"));
}

#[test]
fn consumer_fixture_covers_the_complete_nondynamic_runtime_boundary() {
    for witness in [
        "realm-local intrinsic identity",
        "label + \" writable\"",
        "label + \" enumerable\"",
        "label + \" configurable\"",
        "undefined receiver",
        "number candidate",
        "positive chain",
        "negative chain",
        "bound target recursion",
        "bound custom handler result",
        "call-only function starts without prototype",
        "poisoned prototype abrupt",
        "default prototype stays non-configurable",
        "configurable prototype changes kind",
        "non-object prototype TypeError",
        "Proxy GetPrototypeOf abrupt",
        "Proxy chain abrupt",
    ] {
        assert!(
            FIXTURE.contains(witness),
            "missing fixture witness: {witness}"
        );
    }
    assert!(FIXTURE.contains("__lilaCreateRealm"));
    assert!(CONTRACT.contains("The focused Test262 directory contains eleven files."));
    assert!(CONTRACT.contains("Dynamic Function source"));
    assert!(CONTRACT.contains("generation remains a non-claim"));
}

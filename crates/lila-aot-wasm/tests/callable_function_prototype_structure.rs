const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const FUNCTION_BODY_SOURCE: &str = include_str!("../src/builtins/function.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const MODULE_SOURCE: &str = include_str!("../src/module.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CATALOG_SOURCE: &str = include_str!("../../lila-ir/src/builtins/catalog.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_created_realm_builtin_function_prototypes.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/callable-function-prototype.md");

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
    assert!(earlier < later);
}

#[test]
fn function_prototype_is_one_rooted_nonconstructable_builtin_body() {
    let catalog = bounded(
        CATALOG_SOURCE,
        "    FunctionPrototype {",
        "\n}\n\nimpl StandardBuiltinId {",
    );
    assert!(catalog.contains("=> BUILTIN_FUNCTION_PROTOTYPE_FUNCTION_ID"));
    assert!(catalog.contains("debug: \"%Function.prototype%\""));
    assert!(catalog.contains("flags: []"));
    assert!(catalog.contains("installer: None"));
    assert!(catalog.contains("native: \"\""));

    let function_roots = bounded(
        PLANNING_SOURCE,
        "        if builtin == StandardBuiltinId::FunctionConstructor {",
        "        if builtin == StandardBuiltinId::DisposableStackConstructor {",
    );
    assert_eq!(
        function_roots
            .matches("self.require_standard_builtin(StandardBuiltinId::FunctionPrototype)")
            .count(),
        1
    );

    let zero_length = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn standard_builtin_length(builtin: StandardBuiltinId) -> u64 {",
        "pub(crate) fn host_builtin_length(builtin: HostBuiltinId) -> u64 {",
    );
    assert!(zero_length.contains("| StandardBuiltinId::FunctionPrototype"));

    let dispatch = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::FunctionPrototype => {",
        "            StandardBuiltinId::FunctionPrototypeCall => {",
    );
    assert_eq!(
        dispatch
            .matches("self.emit_function_builtin(FunctionBuiltin::Prototype, function)?")
            .count(),
        1
    );

    let body = bounded(
        FUNCTION_BODY_SOURCE,
        "            FunctionBuiltin::Prototype => {",
        "            FunctionBuiltin::PrototypeCall => {",
    );
    assert!(body.contains("self.emit_undefined_payload(function)"));
    assert!(body.contains("ValueKind::Undefined.tag()"));
    assert!(!body.contains("this_payload_local"));
    assert!(!body.contains("emit_builtin_arg"));
    assert!(!body.contains("emit_alloc"));
}

#[test]
fn entry_realm_materializes_the_callable_over_its_object_prototype() {
    let roots = bounded(
        BOOTSTRAP_SOURCE,
        "    pub(crate) fn init_runtime_roots(&mut self, function: &mut Function)",
        "        // Array.prototype is itself an Array exotic object.",
    );
    assert_eq!(
        roots
            .matches("StandardBuiltinId::FunctionPrototype.function_id()")
            .count(),
        1
    );
    assert_eq!(
        roots
            .matches("self.emit_function_value_payload(&function_prototype_meta, function)?")
            .count(),
        1
    );
    assert_eq!(
        roots
            .matches("Instruction::GlobalSet(FUNCTION_PROTOTYPE_GLOBAL_INDEX)")
            .count(),
        1
    );
    assert!(roots.contains("Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX)"));
    assert!(roots.contains("HEAP_PROTOTYPE_OFFSET"));
    assert!(roots.contains("HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET"));
    assert!(roots.contains("ValueKind::Object.tag()"));
    assert_before(
        roots,
        "self.emit_function_value_payload(&function_prototype_meta, function)?",
        "Instruction::GlobalSet(FUNCTION_PROTOTYPE_GLOBAL_INDEX)",
    );

    let constructor = bounded(
        BOOTSTRAP_SOURCE,
        "    pub(crate) fn init_builtin_constructor_object(",
        "    pub(crate) fn init_throw_type_error_intrinsic(",
    );
    let prototype_kind = bounded(
        constructor,
        "                let prototype_kind = match builtin {",
        "                self.store_i64_const_at_offset(",
    );
    assert!(
        prototype_kind.contains("StandardBuiltinId::FunctionConstructor => ValueKind::Function")
    );
    assert!(!prototype_kind.contains("StandardBuiltinId::FunctionPrototype"));
}

#[test]
fn exact_builtin_identity_uses_the_canonical_intrinsic_global() {
    let global_mapping = bounded(
        MODULE_SOURCE,
        "pub(crate) fn standard_builtin_function_global_index(",
        "/// The canonical, already-allocated function object",
    );
    assert!(global_mapping
        .contains("StandardBuiltinId::FunctionPrototype => Some(FUNCTION_PROTOTYPE_GLOBAL_INDEX)"));

    let identity = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_function_identity_payload(",
        "    /// Emits parameter zero for a standard builtin call or function object.",
    );
    assert_eq!(
        identity
            .matches("preallocated_function_value_global_index(function_id)")
            .count(),
        1
    );
    assert!(!identity.contains("StandardBuiltinId::FunctionPrototype"));

    let allocation = bounded(
        FUNCTIONS_SOURCE,
        "        if let Some(prototype_global_index) =\n            syntax_function_object_prototype_global_index",
        "        let instance_prototype_global_index =",
    );
    assert!(allocation.contains("HEAP_PROTOTYPE_OFFSET"));
    assert!(allocation.contains("HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET"));
    assert!(allocation.contains("ValueKind::Object.tag() as u64"));
}

#[test]
fn callable_prototype_tags_propagate_to_intrinsic_function_prototype_links() {
    let bootstrap = bounded(
        BOOTSTRAP_SOURCE,
        "let callable_function_prototype_local = self.reserve_temp_local();",
        "self.release_temp_local(callable_function_prototype_local);",
    );
    assert_eq!(
        bootstrap
            .matches("self.emit_alloc_plain_object_with_prototype_and_tag(")
            .count(),
        2
    );
    assert!(bootstrap.contains("ValueKind::Function.tag() as i64"));
    assert!(bootstrap.contains("GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX"));
    assert!(bootstrap.contains("ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX"));
}

#[test]
fn created_realm_context_couples_identity_realm_and_object_prototype() {
    let reserved = bounded(
        FUNCTIONS_SOURCE,
        "/// Storage reserved for a created realm's callable `%Function.prototype%`",
        "/// The inseparable realm/default-function-prototype inputs",
    );
    assert!(reserved.contains("#[must_use]"));
    assert!(reserved.contains("struct ReservedRealmFunctionPrototypeLocal(u32)"));
    assert!(!reserved.contains("derive(Clone"));
    assert!(!FUNCTIONS_SOURCE.contains("impl Copy for ReservedRealmFunctionPrototypeLocal"));

    let context = bounded(
        FUNCTIONS_SOURCE,
        "/// The inseparable realm/default-function-prototype inputs",
        "/// Storage reserved for a created realm's `%Array.prototype%`",
    );
    assert!(context.contains("#[must_use]"));
    assert!(context.contains("struct RealmFunctionMaterializationContext"));
    assert!(context.contains("realm: RealmRecordLocal"));
    assert!(context.contains("function_prototype_local: u32"));
    assert!(!context.contains("ValueKind"));

    let initialize = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_initialize_realm_function_materialization_context(",
        "    pub(crate) fn emit_define_realm_function_prototype_data(",
    );
    assert_eq!(
        initialize
            .matches("StandardBuiltinId::FunctionPrototype.function_id()")
            .count(),
        1
    );
    assert_eq!(
        initialize
            .matches("self.emit_function_value_payload(&prototype_meta, function)?")
            .count(),
        1
    );
    assert!(initialize.contains("self.emit_store_function_defining_realm("));
    assert!(initialize.contains("object_prototype_local"));
    assert!(initialize.contains("HEAP_PROTOTYPE_OFFSET"));
    assert!(initialize.contains("HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET"));
    assert!(initialize.contains("ValueKind::Object.tag()"));
    assert_before(
        initialize,
        "self.emit_function_value_payload(&prototype_meta, function)?",
        "self.emit_store_function_defining_realm(",
    );
    assert_before(
        initialize,
        "self.emit_store_function_defining_realm(",
        "Ok(RealmFunctionMaterializationContext",
    );

    let bind = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_bind_realm_function_constructor_prototype(",
        "    pub(crate) fn release_realm_function_materialization_context(",
    );
    assert!(bind.contains("ValueKind::Function.tag()"));
    assert!(bind.contains("HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET"));
    assert!(bind.contains("HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET"));
    assert!(bind.contains(
        "constructor_local,\n            \"prototype\",\n            context.function_prototype_local,\n            tag_local,\n            false,\n            false,\n            false,"
    ));
    assert!(bind.contains(
        "context.function_prototype_local,\n            \"constructor\",\n            constructor_local,\n            tag_local,\n            true,\n            false,\n            true,"
    ));
}

#[test]
fn created_realm_functions_inherit_their_published_callable_prototype() {
    let created_function = bounded(
        FUNCTIONS_SOURCE,
        "    fn emit_function_value_payload_in_realm_with_prototype_materialization(",
        "    pub(crate) fn reserve_realm_function_prototype_local(",
    );
    assert!(created_function.contains("context.function_prototype_local"));
    assert!(created_function.contains("HEAP_PROTOTYPE_OFFSET"));
    assert!(created_function.contains("HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET"));
    assert!(created_function.contains("ValueKind::Function.tag()"));
}

#[test]
fn created_realm_bootstrap_consumes_the_coupled_context_before_publication() {
    let create_realm = bounded(
        HOST_SOURCE,
        "    pub(crate) fn compile_host_create_realm_builtin(",
        "    pub(crate) fn compile_host_realm_eval_script_builtin(",
    );
    assert_eq!(
        create_realm
            .matches("let function_prototype_slot = self.reserve_realm_function_prototype_local()")
            .count(),
        1
    );
    assert_eq!(
        create_realm
            .matches("let realm_functions = self.emit_initialize_realm_function_materialization_context(")
            .count(),
        1
    );
    assert_eq!(
        create_realm
            .matches("self.emit_bind_realm_function_constructor_prototype(")
            .count(),
        1
    );
    assert_eq!(
        create_realm
            .matches("self.release_realm_function_materialization_context(realm_functions)")
            .count(),
        1
    );
    assert_before(
        create_realm,
        "let realm_record = self.emit_alloc_realm_record(",
        "let realm_functions = self.emit_initialize_realm_function_materialization_context(",
    );
    assert_before(
        create_realm,
        "let realm_functions = self.emit_initialize_realm_function_materialization_context(",
        "self.emit_bind_realm_function_constructor_prototype(",
    );
    assert_before(
        create_realm,
        "self.emit_bind_realm_function_constructor_prototype(",
        "self.release_realm_function_materialization_context(realm_functions)",
    );
}

#[test]
fn consumer_oracle_pins_entry_and_created_realm_observables() {
    for witness in [
        "entry Function.prototype",
        "first Function.prototype",
        "second Function.prototype",
        "label + \" typeof\"",
        "label + \" tag\"",
        "function () { [native code] }",
        "label + \" empty call\"",
        "label + \" argument call\"",
        "label + \" length\"",
        "label + \" name\"",
        "label + \" publication\"",
        "label + \" own prototype\"",
        "label + \" construct error\"",
        "Function.prototype realm identity",
    ] {
        assert!(
            FIXTURE.contains(witness),
            "missing fixture witness {witness}"
        );
    }

    for exact in [
        "S15.3.3.1_A1.js",
        "S15.3.4_A1.js",
        "S15.3.4_A2_T1.js",
        "S15.3.4_A2_T2.js",
        "S15.3.4_A2_T3.js",
    ] {
        assert!(CONTRACT.contains(exact), "missing exact witness {exact}");
    }
    assert!(CONTRACT.contains("not a current-HEAD execution result"));
}

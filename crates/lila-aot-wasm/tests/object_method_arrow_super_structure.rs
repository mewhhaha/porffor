const FUNCTION_PROTOCOL_SOURCE: &str = include_str!("../../lila-ir/src/function_protocol.rs");
const ANALYSIS_SOURCE: &str = include_str!("../../lila-ir/src/analysis.rs");
const IR_TEST_SOURCE: &str = include_str!("../../lila-ir/src/lib.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_object_method_arrow_super.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/object-method-arrow-super.md");

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const SELECTED_WITNESSES: [(&str, &str); 2] = [
    witness!("language/expressions/super/prop-dot-obj-val-from-arrow.js"),
    witness!("language/expressions/super/prop-expr-obj-val-from-arrow.js"),
];

const GREEN_CONTROLS: [(&str, &str); 5] = [
    witness!("language/expressions/object/concise-generator.js"),
    witness!("language/expressions/object/method-definition/generator-super-prop-body.js"),
    witness!("language/expressions/object/method-definition/generator-super-prop-param.js"),
    witness!("language/expressions/object/method-definition/async-super-call-body.js"),
    witness!("language/expressions/object/method-definition/async-super-call-param.js"),
];

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
    let earlier_index = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier marker: {earlier}"));
    let later_index = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later marker: {later}"));
    assert!(
        earlier_index < later_index,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn lexical_super_owner_role_is_closed_and_protocol_derived() {
    assert!(FUNCTION_PROTOCOL_SOURCE.contains(
        r#"#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LexicalSuperOwnerRole {
    None,
    HomeObject,
    DerivedConstructorActivation,
}"#
    ));

    let projection = bounded(
        FUNCTION_PROTOCOL_SOURCE,
        "    pub(crate) const fn lexical_super_owner_role(self)",
        "    pub const fn flavor(self)",
    );
    for marker in [
        "Self::ObjectMethod(_)",
        "Self::ObjectGetter",
        "Self::ObjectSetter",
        "Self::ClassMethod(_)",
        "Self::ClassGetter",
        "Self::ClassSetter => LexicalSuperOwnerRole::HomeObject",
        "Self::OrdinaryCallOnly",
        "Self::OrdinaryCallAndConstruct",
        "Self::Arrow",
        "Self::Generator",
        "Self::Async",
        "Self::AsyncArrow",
        "Self::AsyncGenerator",
        "Self::ClassConstructor => LexicalSuperOwnerRole::None",
    ] {
        assert!(projection.contains(marker), "missing projection: {marker}");
    }
    assert!(!projection.contains("_ =>"));

    let owner = bounded(
        ANALYSIS_SOURCE,
        "pub(crate) struct OwnerPlan {",
        "pub(crate) struct FunctionPlan<'a> {",
    );
    assert!(owner.contains("lexical_super_owner_role: LexicalSuperOwnerRole"));
}

#[test]
fn analysis_mints_one_home_object_capability_before_capture_resolution() {
    let parameter_authority = bounded(
        ANALYSIS_SOURCE,
        "    fn collect_parameter_environment_bindings(",
        "    fn activation_binding_modes(",
    );
    for marker in [
        "LexicalSuperOwnerRole::None => {}",
        "LexicalSuperOwnerRole::HomeObject =>",
        "bindings.insert(LEXICAL_HOME_OBJECT_NAME.to_string())",
        "LexicalSuperOwnerRole::DerivedConstructorActivation =>",
        "DERIVED_ACTIVATION_THIS_NAME.to_string()",
        "DERIVED_ACTIVATION_THIS_STATUS_NAME.to_string()",
        "DERIVED_ACTIVATION_NEW_TARGET_NAME.to_string()",
        "DERIVED_ACTIVATION_FUNCTION_NAME.to_string()",
    ] {
        assert!(
            parameter_authority.contains(marker),
            "missing parameter-authority marker: {marker}"
        );
    }
    assert!(!parameter_authority.contains("_ =>"));

    let planning = bounded(
        ANALYSIS_SOURCE,
        "    fn collect_function_plan(",
        "    fn collect_owner_bindings(",
    );
    assert_before(
        planning,
        "let lexical_super_owner_role = function.protocol.lexical_super_owner_role();",
        "self.register_activation_environment(&owner_id, root_bindings.clone())",
    );
    for marker in [
        "self.collect_parameter_environment_bindings(",
        "bindings.insert(LEXICAL_HOME_OBJECT_NAME.to_string())",
        "lexical_super_owner_role,",
    ] {
        assert!(
            planning.contains(marker),
            "missing planning marker: {marker}"
        );
    }
    assert_before(
        planning,
        "let parameter_environment_bindings = self.collect_parameter_environment_bindings(",
        "self.parameter_environment_bindings\n            .insert(owner_id.clone(), parameter_environment_bindings)",
    );
    assert_before(
        planning,
        "bindings.insert(LEXICAL_HOME_OBJECT_NAME.to_string())",
        "self.register_activation_environment(&owner_id, root_bindings.clone())",
    );

    let class_methods = bounded(
        ANALYSIS_SOURCE,
        "    fn collect_class_element_owner_plans(",
        "    fn collect_class_field_initializer_owner_plan(",
    );
    assert!(class_methods.contains("self.collect_parameter_environment_bindings("));
    assert!(class_methods.contains("LexicalSuperOwnerRole::HomeObject"));
    assert_before(
        class_methods,
        "self.collect_parameter_environment_bindings(",
        "self.register_activation_environment(&id, root_bindings.clone())",
    );

    let class_field = bounded(
        ANALYSIS_SOURCE,
        "    fn collect_class_field_initializer_owner_plan(",
        "    fn collect_class_static_block_owner_plan(",
    );
    assert!(class_field.contains("lexical_super_owner_role: LexicalSuperOwnerRole::HomeObject"));

    let class_static = bounded(
        ANALYSIS_SOURCE,
        "    fn collect_class_static_block_owner_plan(",
        "    fn collect_class_constructor_owner_plan(",
    );
    assert!(class_static.contains("lexical_super_owner_role: LexicalSuperOwnerRole::HomeObject"));

    let constructor = bounded(
        ANALYSIS_SOURCE,
        "    fn collect_class_constructor_owner_plan(",
        "    fn collect_default_class_constructor_owner_plan(",
    );
    for marker in [
        "let lexical_super_owner_role = if is_derived_constructor",
        "LexicalSuperOwnerRole::DerivedConstructorActivation",
        "LexicalSuperOwnerRole::HomeObject",
        "self.collect_parameter_environment_bindings(",
        "lexical_super_owner_role,",
    ] {
        assert!(
            constructor.contains(marker),
            "missing constructor marker: {marker}"
        );
    }
    assert_before(
        constructor,
        "self.collect_parameter_environment_bindings(",
        "self.register_activation_environment(&id, root_bindings.clone())",
    );

    let capture = bounded(
        ANALYSIS_SOURCE,
        "    fn record_lexical_super_property_refs(",
        "    fn record_derived_activation_refs(",
    );
    for marker in [
        "match self.lexical_super_owner_role(owner_id)",
        "LexicalSuperOwnerRole::None => {}",
        "LexicalSuperOwnerRole::HomeObject =>",
        "[LEXICAL_THIS_NAME, LEXICAL_HOME_OBJECT_NAME]",
        "LexicalSuperOwnerRole::DerivedConstructorActivation =>",
        "self.record_derived_activation_binding_refs",
    ] {
        assert!(capture.contains(marker), "missing capture marker: {marker}");
    }
    assert!(!capture.contains("class_execution_ids"));
    assert!(!capture.contains("_ =>"));

    let ancestry = bounded(
        ANALYSIS_SOURCE,
        "    fn lexical_super_owner_role(&self, owner_id: &str)",
        "    fn resolve_capture_environment(",
    );
    assert!(ancestry.contains("if owner.flavor != FunctionFlavor::Arrow"));
    assert!(ancestry.contains("return owner.lexical_super_owner_role"));
    assert!(ancestry.contains("current = owner.parent_owner_id.as_deref()"));
    assert!(!ancestry.contains("class_execution_ids"));
}

#[test]
fn existing_backend_stores_and_consumes_the_captured_home_object() {
    let compile = bounded(
        EMIT_SOURCE,
        "    fn compile(&mut self) -> Result<Function, EmitError> {",
        "    fn init_template_objects(",
    );
    assert_before(
        compile,
        "self.init_current_env(&mut function)?",
        "self.bind_parameters(&mut function)?",
    );

    let init = bounded(
        EMIT_SOURCE,
        "    fn init_current_env(&mut self, function: &mut Function)",
        "    fn initialize_derived_activation(",
    );
    let home_object_store = bounded(
        init,
        "            if let Some(slot) = self.owned_env_slot(LEXICAL_HOME_OBJECT_NAME) {",
        "            }\n        }\n        if let Some((activation_local, environment_offset))",
    );
    for marker in [
        "HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET",
        "HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET",
        "BindingStorage::EnvSlot { slot, hops: 0 }",
        "self.write_binding_from_locals(",
    ] {
        assert!(
            home_object_store.contains(marker),
            "missing environment-store marker: {marker}"
        );
    }
    assert_before(
        home_object_store,
        "HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET",
        "self.write_binding_from_locals(",
    );

    let load = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_load_super_base(",
        "    fn emit_load_super_base_from_home_object(",
    );
    assert_before(
        load,
        "current_function_meta()",
        "if self.lexical_derived_activation.is_some()",
    );
    assert_before(
        load,
        "if self.lexical_derived_activation.is_some()",
        "let Some(home_object) = self.lookup_binding(LEXICAL_HOME_OBJECT_NAME)",
    );
    for marker in [
        "self.read_binding_to_locals(",
        "home_object,",
        "self.emit_load_super_base_from_home_object(",
    ] {
        assert!(
            load.contains(marker),
            "missing captured-load marker: {marker}"
        );
    }

    let read = bounded(
        EXPRESSIONS_SOURCE,
        "    fn compile_super_property_read_to_locals(",
        "    fn compile_super_property_write_to_locals(",
    );
    assert_before(
        read,
        "self.compile_expr_to_locals(",
        "self.emit_load_super_base(",
    );
    assert_before(
        read,
        "self.emit_load_super_base(",
        "self.emit_object_read_with_key_tag(",
    );
    assert!(read.contains("receiver_payload_local"));
    assert!(read.contains("receiver_tag_local"));
}

#[test]
fn exact_witnesses_controls_fixture_and_nonclaims_are_pinned() {
    assert_eq!(SELECTED_WITNESSES.len(), 2);
    for (path, source) in SELECTED_WITNESSES {
        assert!(path.contains("language/expressions/super/"));
        assert!(source.contains("super.") || source.contains("super["));
        assert!(source.contains("=>"));
        assert!(!source.contains("flags: [module]"));
    }

    assert_eq!(GREEN_CONTROLS.len(), 5);
    assert!(GREEN_CONTROLS
        .iter()
        .any(|(path, _)| path.ends_with("concise-generator.js")));
    assert_eq!(
        GREEN_CONTROLS
            .iter()
            .filter(|(path, _)| path.contains("generator-super-prop-"))
            .count(),
        2
    );

    let ir_test = bounded(
        IR_TEST_SOURCE,
        "    fn object_method_arrow_super_captures_paired_home_object_authority()",
        "    #[test]\n    fn exact_context_specialization_preserves_escaped_closure_environment()",
    );
    for marker in [
        "function.protocol.is_object_literal_method()",
        "[LEXICAL_THIS_NAME, LEXICAL_HOME_OBJECT_NAME]",
        "assert_eq!(arrows.len(), 4)",
        "assert!(lexical_super_arrows.len() >= 3)",
        "assert!(captured.contains(LEXICAL_THIS_NAME))",
        "assert!(captured.contains(LEXICAL_HOME_OBJECT_NAME))",
    ] {
        assert!(
            ir_test.contains(marker),
            "missing focused IR marker: {marker}"
        );
    }
    assert_eq!(
        GREEN_CONTROLS
            .iter()
            .filter(|(path, _)| path.contains("async-super-call-"))
            .count(),
        2
    );

    for marker in [
        "return () => super.named",
        "return () => super[observeKey()]",
        "return () => () => super.named",
        "factory = () => super.named",
        "object.namedArrow.call(alien)",
        "Object.setPrototypeOf(object, prototypeB)",
        "firstNamed === \"A:alien\"",
        "secondNamed === \"B:alien\"",
        "keyTrace === \"keykey\"",
    ] {
        assert!(FIXTURE.contains(marker), "missing fixture marker: {marker}");
    }

    for marker in [
        "two physical files and four sloppy/strict Script executions",
        "The current Wasm-AOT binary reports `0/4`",
        "object/concise-generator.js` is `2/2`",
        "generator-super-prop-*` files are `4/4`",
        "async-super-call-{body,param}.js` files are `4/4`",
        "does not claim `super()`",
        "does not claim Super Reference numeric/compound update closure",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker: {marker}"
        );
    }
}

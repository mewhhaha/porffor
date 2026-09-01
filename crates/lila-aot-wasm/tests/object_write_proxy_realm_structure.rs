const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const SET_PATH_REALM_SOURCE: &str = include_str!("../src/objects/set_path_realm.rs");
const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const REFLECT_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_set_error_realm.js");
const ARRAY_INHERITED_INDEX_SET_STATE_CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-inherited-index-set-state.md");
const ARRAY_TASK: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn object_mutation_realm_source_exhaustively_projects_every_helper_body() {
    let source_domain = bounded(
        EMIT_SOURCE,
        "pub(crate) enum ObjectMutationErrorRealmSource {",
        "impl ObjectReadErrorRealmSource",
    );
    for state in [
        "GlobalFallback",
        "StandardBuiltinEnvironment",
        "SetPathHelperArgument",
    ] {
        assert!(
            source_domain.contains(state),
            "missing Realm source state: {state}"
        );
    }
    let helper_projection = bounded(
        source_domain,
        "pub(crate) const fn for_runtime_helper(helper: RuntimeHelperId) -> Self {",
        "\n    }\n}",
    );
    assert_eq!(
        helper_projection
            .matches("=> Self::SetPathHelperArgument")
            .count(),
        1
    );
    assert!(helper_projection.contains(
        "RuntimeHelperId::ObjectWrite\n            | RuntimeHelperId::OrdinarySetDataOnReceiver\n            | RuntimeHelperId::OrdinarySetDataOnReceiverWithFallback\n            | RuntimeHelperId::OrdinarySet\n            | RuntimeHelperId::OrdinarySetWithoutReceiverFallback => Self::SetPathHelperArgument"
    ));
    assert!(!helper_projection.contains("_ =>"));

    let helper_entry = bounded(
        EMIT_SOURCE,
        "pub(crate) fn begin_helper_body(&mut self, helper: RuntimeHelperId) -> Function {",
        "Function::new_with_locals_types",
    );
    assert_eq!(
        helper_entry
            .matches("ObjectMutationErrorRealmSource::for_runtime_helper(helper)")
            .count(),
        1
    );
}

#[test]
fn outlined_and_inline_proxy_sets_consume_only_the_typed_realm_projection() {
    for projection in [
        "fn set_path_realm_environment_argument(",
        "fn object_mutation_error_realm(",
    ] {
        let body = bounded(SET_PATH_REALM_SOURCE, projection, "\n}");
        for state in [
            "ObjectMutationErrorRealmSource::GlobalFallback",
            "ObjectMutationErrorRealmSource::StandardBuiltinEnvironment",
            "ObjectMutationErrorRealmSource::SetPathHelperArgument",
        ] {
            assert_eq!(body.matches(state).count(), 1, "{projection}: {state}");
        }
        assert!(!body.contains("_ =>"));
    }

    let helper_body = bounded(
        EMIT_SOURCE,
        "fn compile_object_write_helper(&mut self)",
        "fn compile_ordinary_set_data_on_receiver_helper",
    );
    assert!(helper_body.contains("Instruction::LocalGet(6)"));
    assert!(helper_body.contains("Instruction::LocalSet(self.current_env_local)"));

    let helper_call = bounded(
        OBJECTS_SOURCE,
        "fn emit_object_write_via_helper(",
        "fn ambient_object_write_strict_flag_word",
    );
    assert_eq!(
        helper_call
            .matches("self.emit_set_path_realm_environment_argument(function);")
            .count(),
        1
    );
    assert!(!helper_call.contains("StandardBuiltinId::from_function_id"));

    let builtin_realm_argument = bounded(
        FUNCTIONS_SOURCE,
        "fn emit_standard_builtin_realm_env_argument(",
        "pub(crate) fn emit_function_value_payload(",
    );
    assert_eq!(
        builtin_realm_argument
            .matches("self.emit_set_path_realm_environment_argument(function);")
            .count(),
        1
    );
    assert!(!builtin_realm_argument.contains("StandardBuiltinId::from_function_id"));

    let receiver_helper_call = bounded(
        OBJECTS_SOURCE,
        "fn emit_ordinary_set_data_on_receiver_via_helper(",
        "pub(crate) fn emit_ordinary_set_result(",
    );
    assert_eq!(
        receiver_helper_call
            .matches("self.emit_set_path_realm_environment_argument(function);")
            .count(),
        1
    );
    assert!(!receiver_helper_call.contains("self.current_env_local,"));

    let ordinary_set_helper_call = bounded(
        OBJECTS_SOURCE,
        "fn emit_ordinary_set_result_via_selected_helper(",
        "pub(crate) fn emit_ordinary_set_result_with_receiver_fallback(",
    );
    assert_eq!(
        ordinary_set_helper_call
            .matches("self.emit_set_path_realm_environment_argument(function);")
            .count(),
        1
    );
    assert!(
        ordinary_set_helper_call.contains("(realm_environment_local, realm_environment_tag_local)")
    );
    assert!(
        !ordinary_set_helper_call.contains("(self.current_env_local, realm_environment_tag_local)")
    );

    let realm_error = bounded(
        OBJECTS_SOURCE,
        "fn emit_object_mutation_type_error(",
        "fn emit_object_mutation_type_error_without_message(",
    );
    assert_eq!(
        realm_error
            .matches("object_mutation_error_realm(self.object_mutation_error_realm_source())")
            .count(),
        1
    );
    assert_eq!(
        realm_error
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert_eq!(realm_error.matches("emit_throw_runtime_error(").count(), 1);
}

#[test]
fn array_inherited_index_set_state_is_one_capability_free_code_authority() {
    let variants = bounded(
        ARRAY_SOURCE,
        "pub(crate) enum ArrayInheritedIndexSetState {",
        "\n}\n\nimpl ArrayInheritedIndexSetState",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "Unhandled,",
            "Setter,",
            "OrdinaryRejected,",
            "Handled,",
            "ProxyRejected,",
        ]
    );

    let declaration_offset = ARRAY_SOURCE
        .find("pub(crate) enum ArrayInheritedIndexSetState {")
        .expect("Array inherited index Set state declaration");
    assert_eq!(
        ARRAY_SOURCE[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("}")
    );
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(!ARRAY_SOURCE.contains(&format!(
            "impl {capability} for ArrayInheritedIndexSetState"
        )));
    }

    let code_projection = without_whitespace(bounded(
        ARRAY_SOURCE,
        "impl ArrayInheritedIndexSetState {",
        "\n}\n\n/// Element method and receiver locals",
    ));
    assert_eq!(
        code_projection,
        concat!(
            "pub(crate)constfncode(&self)->i64{matchself{",
            "Self::Unhandled=>0,Self::Setter=>1,Self::OrdinaryRejected=>2,",
            "Self::Handled=>3,Self::ProxyRejected=>4,}}"
        )
    );
    assert!(!code_projection.contains("_=>"));

    assert_eq!(
        ARRAY_SOURCE.matches("ArrayInheritedIndexSetState").count(),
        16
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("ArrayInheritedIndexSetState")
            .count(),
        3
    );
    for (variant, count) in [
        ("Unhandled", 2),
        ("Setter", 3),
        ("OrdinaryRejected", 6),
        ("Handled", 2),
        ("ProxyRejected", 3),
    ] {
        assert_eq!(
            ARRAY_SOURCE
                .matches(&format!("ArrayInheritedIndexSetState::{variant}.code()"))
                .count()
                + STANDARD_SOURCE
                    .matches(&format!("ArrayInheritedIndexSetState::{variant}.code()"))
                    .count(),
            count,
            "{variant}"
        );
    }
    assert_eq!(
        ARRAY_SOURCE
            .matches("self.emit_array_inherited_index_set_state(")
            .count()
            + STANDARD_SOURCE
                .matches("self.emit_array_inherited_index_set_state(")
                .count(),
        2
    );
    for evidence in [ARRAY_INHERITED_INDEX_SET_STATE_CONTRACT, ARRAY_TASK] {
        assert!(evidence.contains("capability-free `ArrayInheritedIndexSetState`"));
    }
}

#[test]
fn proxy_set_type_error_owner_and_consumers_are_a_closed_census() {
    let unconditional_false_result = bounded(
        OBJECTS_SOURCE,
        "fn emit_proxy_set_false_result_throw(",
        "fn emit_assignment_proxy_set_false_result_if_strict(",
    );
    assert_eq!(
        unconditional_false_result
            .matches("emit_object_mutation_type_error(\"Proxy set trap returned false\"")
            .count(),
        1
    );
    assert_eq!(
        unconditional_false_result
            .matches("emit_return_current_completion(function)")
            .count(),
        1
    );

    let assignment_false_result = bounded(
        OBJECTS_SOURCE,
        "fn emit_assignment_proxy_set_false_result_if_strict(",
        "fn emit_object_write_non_extensible_failure(",
    );
    assert_eq!(
        assignment_false_result
            .matches("self.emit_proxy_set_false_result_throw(function)?;")
            .count(),
        2
    );

    let object_write = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_object_write(",
        "pub(crate) fn emit_is_object_entry_backed_tag_i32(",
    );
    assert_eq!(
        object_write
            .matches("self.emit_assignment_proxy_set_false_result_if_strict(function)?;")
            .count(),
        3
    );
    assert!(!object_write.contains("self.emit_proxy_set_false_result_throw(function)?;"));
    assert_eq!(
        object_write
            .matches("self.emit_object_mutation_type_error(\"Proxy handler is null\", function)?;")
            .count(),
        1
    );
    assert_eq!(
        object_write
            .matches(
                "self.emit_object_mutation_type_error(\"Proxy set trap is not callable\", function)?;"
            )
            .count(),
        1
    );
    assert_eq!(
        object_write
            .matches("self.emit_proxy_set_invariant_check(")
            .count(),
        1
    );

    let inherited_index_set = bounded(
        ARRAY_SOURCE,
        "pub(crate) fn emit_array_inherited_index_set_state(",
        "pub(crate) fn emit_string_index_read(",
    );
    assert_eq!(
        inherited_index_set
            .matches("ArrayInheritedIndexSetState::ProxyRejected.code()")
            .count(),
        1
    );

    assert_eq!(
        ARRAY_SOURCE
            .matches("self.emit_array_inherited_index_set_state(")
            .count(),
        1
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("self.emit_array_inherited_index_set_state(")
            .count(),
        1
    );

    let assignment_consumer = bounded(
        ARRAY_SOURCE,
        "pub(crate) fn emit_array_assignment_write(",
        "pub(crate) fn emit_array_inherited_index_set_state(",
    );
    assert_eq!(
        assignment_consumer
            .matches("ArrayInheritedIndexSetState::ProxyRejected.code()")
            .count(),
        1
    );
    assert_eq!(
        assignment_consumer
            .matches("self.emit_assignment_proxy_set_false_result_if_strict(function)?;")
            .count(),
        1
    );
    assert!(!assignment_consumer.contains("self.emit_proxy_set_false_result_throw(function)?;"));

    let push_consumer = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::ArrayPrototypePush => {",
        "StandardBuiltinId::ArrayPrototypeShift => {",
    );
    assert_eq!(
        push_consumer
            .matches("ArrayInheritedIndexSetState::ProxyRejected.code()")
            .count(),
        1
    );
    assert_eq!(
        push_consumer
            .matches("self.emit_proxy_set_false_result_throw(function)?;")
            .count(),
        1
    );
    assert!(!push_consumer
        .contains("self.emit_assignment_proxy_set_false_result_if_strict(function)?;"));

    let invariant = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_proxy_set_invariant_check(",
        "pub(crate) fn emit_object_delete_ordinary(",
    );
    assert_eq!(
        invariant
            .matches("self.emit_object_mutation_type_error(")
            .count(),
        2
    );
    assert_eq!(
        REFLECT_SOURCE
            .matches("self.emit_proxy_set_invariant_check(")
            .count(),
        1
    );

    let reflect_set = bounded(
        REFLECT_SOURCE,
        "pub(crate) fn compile_reflect_set_builtin(",
        "pub(crate) fn compile_reflect_has_builtin(",
    );
    assert_eq!(
        reflect_set
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        2
    );
    assert_eq!(
        reflect_set
            .matches("self.emit_load_live_proxy_slots(")
            .count(),
        1
    );
    assert_eq!(
        reflect_set
            .matches("ProxyRevocationRoute::CurrentFunctionRealm")
            .count(),
        1
    );
    assert_eq!(reflect_set.matches("\"Proxy handler is null\"").count(), 0);
    assert_eq!(
        reflect_set
            .matches("\"Proxy set trap is not callable\"")
            .count(),
        1
    );
}

#[test]
fn borrowed_created_realm_builtins_pin_proxy_set_error_realms() {
    for marker in [
        "other.Array.prototype.fill.call(revocable.proxy, 1)",
        "other.Array.prototype.fill.call(nonCallableProxy, 1)",
        "other.Array.prototype.fill.call(falseResultProxy, 1)",
        "other.Reflect.set(incompatibleProxy, \"fixed\", 2)",
        "other.Reflect.set(directReflectRevocable.proxy, \"value\", 1)",
        "other.Reflect.set(directReflectNonCallable, \"value\", 1)",
        "other.Array.prototype.fill.call(fillPrototypeReceiver, 2)",
        "other.Array.prototype.fill.call(falsePrototypeReceiver, 2)",
        "other.Array.prototype.push.call(pushFalsePrototypeReceiver, 2)",
        "other.Reflect.set(reflectPrototypeReceiver, \"value\", 1)",
        "Object.getPrototypeOf(error) === other.TypeError.prototype",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(CLI_TESTS.contains("fn proxy_set_errors_use_the_borrowed_builtin_realm()"));
    assert!(CLI_TESTS.contains("wasm_proxy_set_error_realm.js"));
}

const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const MAIN_REALM_SOURCE: &str = include_str!("../src/intrinsics/binary_data.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/data_view.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_dataview_created_realm_prototype.js");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/created-realm-data-view-publication-lifecycle.md"
);
const TASK: &str = include_str!("../../../tasks/17-typedarrays-binary-data-atomics.md");

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    let end_offset = source[start_offset..]
        .find(end)
        .map(|offset| start_offset + offset)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"));
    &source[start_offset..end_offset]
}

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn publication_plan() -> &'static str {
    HOST_SOURCE
        .split_once("const CREATED_REALM_DATA_VIEW_PROTOTYPE_PUBLICATIONS:")
        .expect("created-Realm DataView publication plan")
        .1
        .split_once("fn created_realm_string_prototype_method_aliases")
        .expect("created-Realm DataView publication plan end")
        .0
}

fn publication_installer() -> &'static str {
    HOST_SOURCE
        .split_once("        for publication in CREATED_REALM_DATA_VIEW_PROTOTYPE_PUBLICATIONS {")
        .expect("created-Realm DataView publication installer")
        .1
        .split_once(
            "        self.emit_function_value_payload_in_realm(\n            &function_meta,\n            &realm_functions,\n            typed_array_constructor_local,",
        )
        .expect("created-Realm DataView publication installer end")
        .0
}

#[test]
fn created_realm_data_view_plan_is_complete_and_matches_main_realm_order() {
    let plan = publication_plan();
    let main_realm = MAIN_REALM_SOURCE
        .split_once("    pub(crate) fn install_data_view_constructor_intrinsics(")
        .expect("main-Realm DataView installer")
        .1;
    let expected = [
        "DataViewPrototypeBufferGetter",
        "DataViewPrototypeByteLengthGetter",
        "DataViewPrototypeByteOffsetGetter",
        "DataViewPrototypeGetUint8",
        "DataViewPrototypeSetUint8",
        "DataViewPrototypeGetInt8",
        "DataViewPrototypeSetInt8",
        "DataViewPrototypeGetUint16",
        "DataViewPrototypeSetUint16",
        "DataViewPrototypeGetInt16",
        "DataViewPrototypeSetInt16",
        "DataViewPrototypeGetUint32",
        "DataViewPrototypeSetUint32",
        "DataViewPrototypeGetInt32",
        "DataViewPrototypeSetInt32",
        "DataViewPrototypeGetFloat16",
        "DataViewPrototypeSetFloat16",
        "DataViewPrototypeGetFloat32",
        "DataViewPrototypeSetFloat32",
        "DataViewPrototypeGetFloat64",
        "DataViewPrototypeSetFloat64",
        "DataViewPrototypeGetBigInt64",
        "DataViewPrototypeSetBigInt64",
        "DataViewPrototypeGetBigUint64",
        "DataViewPrototypeSetBigUint64",
    ];

    assert_eq!(
        plan.matches("        builtin: StandardBuiltinId::").count(),
        expected.len()
    );
    assert_eq!(plan.matches("        property_kind: Accessor,").count(), 3);
    assert_eq!(plan.matches("        property_kind: Method,").count(), 22);
    assert_eq!(plan.matches("    ToStringTag,").count(), 1);
    assert_eq!(
        main_realm
            .matches(".property_key_symbol_payload(\"Symbol.toStringTag\")")
            .count(),
        1
    );
    assert_eq!(
        main_realm
            .matches("self.strings.payload(\"DataView\")")
            .count(),
        1
    );

    let mut previous_plan_offset = 0;
    let mut previous_main_realm_offset = 0;
    for builtin in expected {
        let reference = format!("StandardBuiltinId::{builtin}");
        assert_eq!(
            plan.matches(&reference).count(),
            1,
            "created-Realm plan must contain `{builtin}` exactly once"
        );
        assert_eq!(
            main_realm.matches(&reference).count(),
            1,
            "main-Realm installer must contain `{builtin}` exactly once"
        );
        let plan_offset = plan.find(&reference).expect("checked plan entry");
        let main_realm_offset = main_realm
            .find(&reference)
            .expect("checked main-Realm entry");
        assert!(
            plan_offset >= previous_plan_offset,
            "created-Realm `{builtin}` order"
        );
        assert!(
            main_realm_offset >= previous_main_realm_offset,
            "main-Realm `{builtin}` order"
        );
        previous_plan_offset = plan_offset;
        previous_main_realm_offset = main_realm_offset;
    }
}

#[test]
fn created_realm_data_view_plan_has_one_move_only_publication_lifecycle() {
    let declarations = without_whitespace(bounded_source(
        HOST_SOURCE,
        "mod html_dda;",
        "const CREATED_REALM_DATA_VIEW_PROTOTYPE_PUBLICATIONS:",
    ));
    assert_eq!(
        declarations,
        concat!(
            "modhtml_dda;",
            "enumCreatedRealmDataViewPropertyKind{Accessor,Method,}",
            "enumCreatedRealmDataViewPrototypePublication{",
            "Callable{builtin:StandardBuiltinId,",
            "property_kind:CreatedRealmDataViewPropertyKind,},ToStringTag,}",
            "useCreatedRealmDataViewPropertyKind::{Accessor,Method};",
            "useCreatedRealmDataViewPrototypePublication::{Callable,ToStringTag};"
        ),
        "both publication authorities must remain exact and attribute-free"
    );
    assert_eq!(
        HOST_SOURCE
            .matches("CreatedRealmDataViewPropertyKind")
            .count(),
        3
    );
    assert_eq!(
        HOST_SOURCE
            .matches("CreatedRealmDataViewPrototypePublication")
            .count(),
        3
    );
    for authority in [
        "CreatedRealmDataViewPropertyKind",
        "CreatedRealmDataViewPrototypePublication",
    ] {
        for capability in [
            "Clone",
            "Copy",
            "Debug",
            "Default",
            "PartialEq",
            "Eq",
            "Hash",
            "PartialOrd",
            "Ord",
        ] {
            assert!(!HOST_SOURCE.contains(&format!("impl {capability} for {authority}")));
        }
        assert!(!HOST_SOURCE.contains(&format!("type {authority}")));
        assert!(!HOST_SOURCE.contains(&format!("{authority}::clone(")));
    }

    let plan = without_whitespace(bounded_source(
        HOST_SOURCE,
        "const CREATED_REALM_DATA_VIEW_PROTOTYPE_PUBLICATIONS:",
        "fn created_realm_string_prototype_method_aliases",
    ));
    assert_eq!(plan.matches("Callable{").count(), 25);
    assert_eq!(plan.matches("property_kind:Accessor,").count(), 3);
    assert_eq!(plan.matches("property_kind:Method,").count(), 22);
    assert_eq!(plan.matches("ToStringTag,").count(), 1);
    assert_eq!((plan.len(), fnv1a(&plan)), (2292, 0x5a40_c486_262b_dc45));
    assert_eq!(
        HOST_SOURCE
            .matches("for publication in CREATED_REALM_DATA_VIEW_PROTOTYPE_PUBLICATIONS")
            .count(),
        1
    );

    let lifecycle = without_whitespace(bounded_source(
        HOST_SOURCE,
        "        for publication in CREATED_REALM_DATA_VIEW_PROTOTYPE_PUBLICATIONS {",
        concat!(
            "        self.emit_function_value_payload_in_realm(\n",
            "            &function_meta,\n",
            "            &realm_functions,\n",
            "            typed_array_constructor_local,"
        ),
    ));
    assert_eq!(lifecycle.matches("publication").count(), 2);
    assert_eq!(lifecycle.matches("matchpublication{").count(), 1);
    assert_eq!(lifecycle.matches("property_kind").count(), 3);
    assert_eq!(lifecycle.matches("match&property_kind{").count(), 2);
    assert_eq!(lifecycle.matches("matchproperty_kind{").count(), 0);
    let first_borrow = lifecycle.find("match&property_kind{").unwrap();
    let second_borrow = lifecycle[first_borrow + 1..]
        .find("match&property_kind{")
        .map(|offset| first_borrow + 1 + offset)
        .unwrap();
    assert!(first_borrow < second_borrow);
    for forbidden in [
        "publication.clone(",
        "property_kind.clone(",
        "&mutproperty_kind",
        "*property_kind",
        "=property_kind",
    ] {
        assert!(!lifecycle.contains(forbidden), "found `{forbidden}`");
    }
    let pre_hardening_semantics = lifecycle.replace("match&property_kind", "matchproperty_kind");
    assert_eq!(
        (
            pre_hardening_semantics.len(),
            fnv1a(&pre_hardening_semantics)
        ),
        (2516, 0x608e_f2c4_a91e_5569)
    );

    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "one move-only publication lifecycle",
        "twenty-six publication rows",
        "two borrowed property-kind decisions",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(task_words.contains(marker), "missing T17 marker: {marker}");
    }
}

#[test]
fn created_realm_data_view_callables_capture_their_realm_before_publication() {
    let installer = publication_installer();

    for binding in [
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert_eq!(
            installer.matches(binding).count(),
            1,
            "DataView callable installer must set {binding} exactly once"
        );
    }
    assert_eq!(
        installer
            .matches("self.emit_function_value_payload_in_realm(")
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches("self.emit_object_define_accessor(")
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches("self.emit_object_define_local_data(")
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches("self.emit_object_define_local_data_with_flags(")
            .count(),
        1
    );
    assert!(installer.contains("self.strings.payload(DATA_VIEW_NAME)"));
    assert!(installer.contains(
        "false,\n                        false,\n                        true,\n                        function,"
    ));
}

#[test]
fn focused_cli_fixture_borrows_created_realm_data_view_methods() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_borrows_created_realm_dataview_prototype_methods()"));
    assert!(CLI_TESTS.contains("wasm_dataview_created_realm_prototype.js"));
    for marker in [
        "borrowed getter positive bound",
        "borrowed setter positive bound",
        "getter method realm identity",
        "setter method realm identity",
        "created realm toStringTag descriptor",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}

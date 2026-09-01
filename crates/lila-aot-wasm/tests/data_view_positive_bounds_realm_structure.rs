const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const BINARY_DATA_SOURCE: &str = include_str!("../src/builtins/binary_data.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/data_view.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_dataview_constructor_range_error_realm.js");
const TYPE_ERROR_CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_dataview_type_error_realm.js");

fn owner_body(start: &str, end: &str) -> &'static str {
    STANDARD_SOURCE
        .split_once(start)
        .unwrap_or_else(|| panic!("missing DataView owner: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing DataView owner boundary: {end}"))
        .0
}

fn data_view_constructor() -> &'static str {
    owner_body(
        "            StandardBuiltinId::DataViewConstructor => {",
        "            StandardBuiltinId::Float64ArrayConstructor",
    )
}

fn data_view_access_owners() -> [(&'static str, &'static str); 10] {
    let boundaries = [
        (
            "getInt8/getUint8",
            "            StandardBuiltinId::DataViewPrototypeGetUint8\n            | StandardBuiltinId::DataViewPrototypeGetInt8 => {",
            "            StandardBuiltinId::DataViewPrototypeGetUint16",
        ),
        (
            "getInt16/getUint16",
            "            StandardBuiltinId::DataViewPrototypeGetUint16\n            | StandardBuiltinId::DataViewPrototypeGetInt16 => {",
            "            StandardBuiltinId::DataViewPrototypeGetUint32",
        ),
        (
            "getInt32/getUint32",
            "            StandardBuiltinId::DataViewPrototypeGetUint32\n            | StandardBuiltinId::DataViewPrototypeGetInt32 => {",
            "            StandardBuiltinId::DataViewPrototypeGetBigInt64",
        ),
        (
            "getBigInt64/getBigUint64",
            "            StandardBuiltinId::DataViewPrototypeGetBigInt64\n            | StandardBuiltinId::DataViewPrototypeGetBigUint64 => {",
            "            StandardBuiltinId::DataViewPrototypeGetFloat16",
        ),
        (
            "floating getters",
            "            StandardBuiltinId::DataViewPrototypeGetFloat16\n            | StandardBuiltinId::DataViewPrototypeGetFloat32",
            "            StandardBuiltinId::DataViewPrototypeSetUint8",
        ),
        (
            "setInt8/setUint8",
            "            StandardBuiltinId::DataViewPrototypeSetUint8\n            | StandardBuiltinId::DataViewPrototypeSetInt8 => {",
            "            StandardBuiltinId::DataViewPrototypeSetUint16",
        ),
        (
            "setInt16/setUint16",
            "            StandardBuiltinId::DataViewPrototypeSetUint16\n            | StandardBuiltinId::DataViewPrototypeSetInt16 => {",
            "            StandardBuiltinId::DataViewPrototypeSetUint32",
        ),
        (
            "setInt32/setUint32",
            "            StandardBuiltinId::DataViewPrototypeSetUint32\n            | StandardBuiltinId::DataViewPrototypeSetInt32 => {",
            "            StandardBuiltinId::DataViewPrototypeSetFloat16",
        ),
        (
            "floating setters",
            "            StandardBuiltinId::DataViewPrototypeSetFloat16\n            | StandardBuiltinId::DataViewPrototypeSetFloat32",
            "            StandardBuiltinId::DataViewPrototypeSetBigInt64",
        ),
        (
            "setBigInt64/setBigUint64",
            "            StandardBuiltinId::DataViewPrototypeSetBigInt64\n            | StandardBuiltinId::DataViewPrototypeSetBigUint64 => {",
            "            StandardBuiltinId::BigIntConstructor => {",
        ),
    ];

    boundaries.map(|(label, start, end)| (label, owner_body(start, end)))
}

fn data_view_private_slot_accessors() -> &'static str {
    owner_body(
        "            StandardBuiltinId::DataViewPrototypeBufferGetter",
        "            StandardBuiltinId::TypedArrayPrototypeToStringTagGetter",
    )
}

fn data_view_current_length_validator() -> &'static str {
    BINARY_DATA_SOURCE
        .split_once("    pub(crate) fn emit_validate_data_view_current_byte_length(")
        .expect("missing DataView current-length validator")
        .1
        .split_once("    pub(crate) fn emit_typed_array_valid_integer_index_i32(")
        .expect("missing DataView current-length validator boundary")
        .0
}

#[test]
fn constructor_routes_conversion_capacity_and_revalidation_bounds_through_its_realm() {
    let body = data_view_constructor();
    assert_eq!(
        body.matches("emit_throw_current_function_realm_range_error(")
            .count(),
        8
    );
    assert!(!body.contains("emit_throw_runtime_error(\n                    RANGE_ERROR_NAME,"));
    assert_eq!(body.matches("DataView byteOffset out of bounds").count(), 4);
    assert_eq!(body.matches("DataView byteLength out of bounds").count(), 4);

    let errors: Vec<_> = body
        .match_indices("emit_throw_current_function_realm_range_error(")
        .map(|(position, _)| position)
        .collect();
    let prototype_read = body
        .find("self.emit_object_read(")
        .expect("missing DataView NewTarget prototype read");
    assert!(errors[5] < prototype_read && prototype_read < errors[6]);
}

#[test]
fn ten_data_view_access_owner_groups_have_one_current_realm_positive_bound() {
    for (label, body) in data_view_access_owners() {
        assert_eq!(
            body.matches("emit_throw_current_function_realm_range_error(")
                .count(),
            1,
            "{label} must own exactly one current-Realm positive bound"
        );
        assert!(
            !body.contains("emit_throw_runtime_error(\n                    RANGE_ERROR_NAME,"),
            "{label} must not retain an entry-global RangeError route"
        );
    }
}

#[test]
fn data_view_algorithm_type_errors_use_the_executing_builtin_realm() {
    let constructor = data_view_constructor();
    assert_eq!(
        constructor
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        3
    );
    assert!(
        !constructor.contains("emit_throw_runtime_error(\n                    TYPE_ERROR_NAME,")
    );

    let validator = data_view_current_length_validator();
    assert_eq!(
        validator
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        3
    );
    assert!(!validator.contains("emit_throw_runtime_error("));

    assert_eq!(
        STANDARD_SOURCE
            .matches("emit_validate_data_view_current_byte_length(")
            .count(),
        11,
        "closed production DataView validator call-site census"
    );
    assert_eq!(
        data_view_private_slot_accessors()
            .matches("emit_validate_data_view_current_byte_length(")
            .count(),
        1
    );
    for (label, body) in data_view_access_owners() {
        assert_eq!(
            body.matches("emit_validate_data_view_current_byte_length(")
                .count(),
            1,
            "{label} must use the current-Realm DataView validator"
        );
    }
}

#[test]
fn created_realm_data_view_constructor_captures_its_error_prototypes() {
    let owner = HOST_SOURCE
        .split_once("            &data_view_meta,")
        .expect("missing created-Realm DataView constructor materialization")
        .1
        .split_once("        for publication in CREATED_REALM_DATA_VIEW_PROTOTYPE_PUBLICATIONS {")
        .expect("missing created-Realm DataView prototype publication boundary")
        .0;

    assert_eq!(owner.matches("HEAP_FUNCTION_ENV_HANDLE_OFFSET").count(), 1);
    assert_eq!(
        owner
            .matches("HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET")
            .count(),
        1
    );
    assert_eq!(
        owner
            .matches("HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET")
            .count(),
        1
    );
    assert!(owner.contains("range_error_prototype_local"));
}

#[test]
fn focused_cli_fixture_pins_all_published_constructor_bound_families() {
    let test = CLI_TESTS
        .split_once("fn run_wasm_backend_uses_borrowed_dataview_constructor_range_error_realm()")
        .expect("missing focused DataView constructor Realm test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused DataView constructor Realm test")
        .0;
    assert!(test.contains("wasm_dataview_constructor_range_error_realm.js"));
    assert!(test.contains("boolean(true)"));

    for marker in [
        "borrowed DataView byteOffset ToIndex realm",
        "borrowed DataView byteOffset capacity realm",
        "borrowed DataView byteLength ToIndex realm",
        "borrowed DataView byteLength capacity realm",
        "borrowed DataView post-prototype byteOffset realm",
        "borrowed DataView post-prototype byteLength realm",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}

#[test]
fn focused_cli_fixture_pins_published_data_view_type_error_families_and_order() {
    assert!(CLI_TESTS.contains("fn run_wasm_backend_uses_borrowed_dataview_type_error_realm()"));
    assert!(CLI_TESTS.contains("wasm_dataview_type_error_realm.js"));
    for marker in [
        "borrowed DataView requires new",
        "borrowed DataView invalid buffer",
        "invalid buffer precedes offset coercion",
        "offset coercion precedes detached constructor check",
        "borrowed DataView post-prototype detachment",
        "borrowed DataView getter invalid receiver",
        "borrowed DataView setter invalid receiver",
        "setter receiver check precedes value coercion",
        "borrowed DataView private-slot getter invalid receiver",
        "borrowed DataView getter detached buffer",
        "index coercion precedes detached method check",
        "borrowed DataView getter out-of-bounds view",
    ] {
        assert!(
            TYPE_ERROR_CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}

const ATOMICS_SOURCE: &str = include_str!("../src/builtins/atomics.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/binary_data.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_atomics_created_realm.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/atomics-builtin-dispatch-boundary.md");
const TASK_T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const TASK_T17: &str = include_str!("../../../tasks/17-typedarrays-binary-data-atomics.md");

fn publication_domain() -> &'static str {
    ATOMICS_SOURCE
        .split_once("pub(super) const ATOMICS_PUBLICATION_ORDER")
        .expect("Atomics publication domain")
        .1
        .split_once("enum AtomicsIntegerOperation")
        .expect("Atomics publication domain end")
        .0
}

fn created_realm_installer() -> &'static str {
    HOST_SOURCE
        .split_once(
            "        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;\n        function.instruction(&Instruction::LocalSet(atomics_object_local));",
        )
        .expect("created-Realm Atomics installer")
        .1
        .split_once(
            "        self.emit_function_value_payload_in_realm(\n            &promise_meta,",
        )
        .expect("created-Realm Atomics installer end")
        .0
}

#[test]
fn atomics_publication_domain_is_closed_and_shared_by_both_realms() {
    let publication = publication_domain();
    let expected = [
        "Add",
        "And",
        "CompareExchange",
        "Exchange",
        "Load",
        "Notify",
        "Or",
        "Pause",
        "Store",
        "Sub",
        "Wait",
        "WaitAsync",
        "Xor",
        "IsLockFree",
    ];

    assert!(ATOMICS_SOURCE
        .contains("pub(super) const ATOMICS_PUBLICATION_ORDER: [StandardBuiltinId; 14]"));
    let mut previous_order_offset = 0;
    for builtin in expected {
        let order_entry = format!("StandardBuiltinId::Atomics{builtin},");
        assert_eq!(
            publication.matches(&order_entry).count(),
            1,
            "publication order must contain Atomics{builtin} exactly once"
        );
        let order_offset = publication.find(&order_entry).expect("checked order entry");
        assert!(
            order_offset >= previous_order_offset,
            "publication order must place Atomics{builtin} after its predecessor"
        );
        previous_order_offset = order_offset;
    }

    for source in [BOOTSTRAP_SOURCE, HOST_SOURCE] {
        assert_eq!(
            source
                .matches("for builtin in ATOMICS_PUBLICATION_ORDER")
                .count(),
            1
        );
        assert!(!source.contains("AtomicsBuiltin"));
        assert!(!source.contains("atomics_standard_builtin"));
        assert!(source.contains("builtin.native_function_name()"));
    }
}

#[test]
fn atomics_dispatch_exposes_only_fixed_family_entries() {
    let builtin_declaration = ATOMICS_SOURCE
        .split_once("mod wait_async_result;\n\n")
        .expect("Atomics builtin declaration start")
        .1
        .split_once("\npub(super) const ATOMICS_PUBLICATION_ORDER")
        .expect("Atomics builtin declaration end")
        .0;
    assert_eq!(
        builtin_declaration,
        "enum AtomicsBuiltin {\n    Add,\n    And,\n    CompareExchange,\n    Exchange,\n    IsLockFree,\n    Load,\n    Notify,\n    Or,\n    Pause,\n    Store,\n    Sub,\n    Wait,\n    WaitAsync,\n    Xor,\n}\n"
    );
    assert!(!builtin_declaration.contains("#[derive("));
    assert!(!builtin_declaration.contains("pub(super) enum AtomicsBuiltin"));
    assert_eq!(
        ATOMICS_SOURCE.matches("fn emit_atomics_builtin(").count(),
        1
    );
    assert!(!ATOMICS_SOURCE.contains("pub(super) fn emit_atomics_builtin("));
    assert!(!STANDARD_SOURCE.contains("AtomicsBuiltin"));
    assert!(!STANDARD_SOURCE.contains("emit_atomics_builtin("));

    for (variant, method) in [
        ("Add", "add"),
        ("And", "and"),
        ("CompareExchange", "compare_exchange"),
        ("Exchange", "exchange"),
        ("IsLockFree", "is_lock_free"),
        ("Load", "load"),
        ("Notify", "notify"),
        ("Or", "or"),
        ("Pause", "pause"),
        ("Store", "store"),
        ("Sub", "sub"),
        ("Wait", "wait"),
        ("WaitAsync", "wait_async"),
        ("Xor", "xor"),
    ] {
        let fixed_entry = format!("pub(super) fn emit_atomics_{method}_builtin(");
        let private_route =
            format!("self.emit_atomics_builtin(AtomicsBuiltin::{variant}, function)");
        let standard_route = format!("self.emit_atomics_{method}_builtin(function)?");
        assert_eq!(
            ATOMICS_SOURCE.matches(&fixed_entry).count(),
            1,
            "Atomics.{variant} must expose one fixed family entry"
        );
        assert_eq!(
            ATOMICS_SOURCE.matches(&private_route).count(),
            1,
            "Atomics.{variant} must enter the private family domain once"
        );
        assert_eq!(
            STANDARD_SOURCE.matches(&standard_route).count(),
            1,
            "Atomics.{variant} must have one fixed catalog route"
        );
    }
}

#[test]
fn created_realm_atomics_methods_capture_their_realm_before_publication() {
    let installer = created_realm_installer();

    assert_eq!(
        installer
            .matches("self.emit_function_value_payload_in_realm(")
            .count(),
        1
    );
    for binding in [
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert_eq!(
            installer.matches(binding).count(),
            1,
            "Atomics method installer must set {binding} exactly once"
        );
    }
    assert!(installer.contains("self.strings.payload(ATOMICS_NAME)"));
    assert!(
        installer.contains("false,\n            false,\n            true,\n            function,")
    );
    assert!(installer.contains("atomics_object_local,\n                property_name,"));
    assert!(HOST_SOURCE
        .contains("global_local,\n            ATOMICS_NAME,\n            atomics_object_local,"));
}

#[test]
fn focused_cli_fixture_borrows_created_realm_atomics_without_waiting() {
    assert!(CLI_TESTS.contains("fn run_wasm_backend_borrows_created_realm_atomics_methods()"));
    assert!(CLI_TESTS.contains("wasm_atomics_created_realm.js"));
    for marker in [
        "created realm Atomics identity",
        "created realm Atomics global descriptor",
        "created realm Atomics toStringTag descriptor",
        "borrowed add result",
        "borrowed add TypeError realm",
        "borrowed add RangeError realm",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
    assert!(CLI_FIXTURE.contains("\"wait\","));
    assert!(!CLI_FIXTURE.contains("otherAtomics.wait("));
    assert!(!CLI_FIXTURE.contains("Atomics.wait("));
}

#[test]
fn atomics_dispatch_contract_records_the_exact_boundary_and_nonclaims() {
    for marker in [
        "private, non-derived selection domain",
        "fourteen catalog cases",
        "ATOMICS_PUBLICATION_ORDER: [StandardBuiltinId; 14]",
        "3382f4b6d98ca6acfb04ad9c9f452bd1f93bf65f9d3334e0cef0f17583366231",
        "source-equivalent compile-time hardening",
        "does not close T17",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker: {marker}"
        );
    }
    for task in [TASK_T02, TASK_T17] {
        assert!(task.contains("atomics-builtin-dispatch-boundary.md"));
        assert!(task.contains("ATOMICS_PUBLICATION_ORDER"));
        assert!(task.contains("5/5"));
        assert!(task.contains("3382f4b6d98ca6acfb04ad9c9f452bd1f93bf65f9d3334e0cef0f17583366231"));
        assert!(task.contains("no new Atomics behavior"));
    }
}

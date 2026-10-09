const FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_proxy_delete_property.js");
const CLI_REGISTRATION: &str = include_str!("../../../lila-cli/tests/cli/object.rs");
const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/proxy-delete-traversal.md");
const TASK: &str = include_str!("../../../../tasks/11-proxy-reflect-metaobject.md");

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const VENDORED_WITNESSES: [(&str, &str); 3] = [
    witness!("built-ins/Proxy/deleteProperty/trap-is-missing-target-is-proxy.js"),
    witness!("built-ins/Proxy/deleteProperty/trap-is-null-target-is-proxy.js"),
    witness!("built-ins/Proxy/deleteProperty/trap-is-undefined-target-is-proxy.js"),
];

#[test]
fn focused_fixture_crosses_six_nullish_proxy_targets_in_order() {
    assert_eq!(
        FIXTURE
            .matches("deepDeleteProxy = new Proxy(deepDeleteProxy, nullishDeleteHandler(")
            .count(),
        6
    );
    for marker in [
        "deepDeleteOrder = deepDeleteOrder * 10 + marker;",
        "deepDeleteOrder = deepDeleteOrder * 10 + 7;",
        "deepDeleteOrder !== 6543217",
        "deepDeleteCalls !== 1",
        "Object.prototype.hasOwnProperty.call(\n  deepDeleteTarget,\n  \"forwardedAcrossSixProxies\"",
    ] {
        assert!(FIXTURE.contains(marker), "missing fixture marker: {marker}");
    }
    assert!(CLI_REGISTRATION
        .contains("fn run_wasm_backend_succeeds_for_supported_proxy_delete_property_fixture()"));
    assert!(CLI_REGISTRATION.contains("fixture_path(\"wasm_proxy_delete_property.js\")"));
    assert!(CONTRACT.contains("six nullish forwarding handlers"));
    assert!(TASK.contains("six nullish forwarding handlers"));
}

#[test]
fn exact_nested_target_witnesses_retain_six_default_executions() {
    assert_eq!(VENDORED_WITNESSES.len(), 3);
    for (path, source) in VENDORED_WITNESSES {
        assert!(path.contains("target-is-proxy"));
        assert!(source.contains("features: [Proxy, Reflect]"));
        for single_mode in [
            "flags: [module]",
            "flags: [noStrict]",
            "flags: [onlyStrict]",
        ] {
            assert!(
                !source.contains(single_mode),
                "{path} must retain two modes"
            );
        }
    }
    assert!(CONTRACT.contains("three physical files and six executions"));
}

const TEST262_SOURCE: &str = include_str!("../../../lila-test262/src/lib.rs");
const FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_object_prevent_extensions_proxy.js");
const CLI_REGISTRATION: &str = include_str!("../../../lila-cli/tests/cli/object.rs");
const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/proxy-prevent-extensions-request.md");

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const VENDORED_WITNESSES: [(&str, &str); 12] = [
    witness!("built-ins/Proxy/preventExtensions/call-parameters.js"),
    witness!("built-ins/Proxy/preventExtensions/null-handler.js"),
    witness!("built-ins/Proxy/preventExtensions/return-false.js"),
    witness!("built-ins/Proxy/preventExtensions/return-is-abrupt.js"),
    witness!("built-ins/Proxy/preventExtensions/return-true-target-is-extensible.js"),
    witness!("built-ins/Proxy/preventExtensions/return-true-target-is-not-extensible.js"),
    witness!("built-ins/Proxy/preventExtensions/trap-is-missing-target-is-proxy.js"),
    witness!("built-ins/Proxy/preventExtensions/trap-is-not-callable-realm.js"),
    witness!("built-ins/Proxy/preventExtensions/trap-is-not-callable.js"),
    witness!("built-ins/Proxy/preventExtensions/trap-is-null-target-is-proxy.js"),
    witness!("built-ins/Proxy/preventExtensions/trap-is-undefined-target-is-proxy.js"),
    witness!("built-ins/Proxy/preventExtensions/trap-is-undefined.js"),
];

#[test]
fn exact_raw_module_witness_and_complete_leaf_inventory_are_retained() {
    assert_eq!(VENDORED_WITNESSES.len(), 12);
    let module_executions = VENDORED_WITNESSES
        .iter()
        .filter(|(_, source)| source.contains("flags: [module]"))
        .count();
    assert_eq!(module_executions, 1);
    assert_eq!(VENDORED_WITNESSES.len() * 2 - module_executions, 23);
    for (path, source) in VENDORED_WITNESSES {
        if source.contains("flags: [module]") {
            assert!(path.ends_with("trap-is-undefined-target-is-proxy.js"));
        } else {
            assert!(!source.contains("flags: [noStrict]"), "{path}");
            assert!(!source.contains("flags: [onlyStrict]"), "{path}");
        }
    }

    let (_, module) = VENDORED_WITNESSES
        .iter()
        .find(|(path, _)| path.ends_with("trap-is-undefined-target-is-proxy.js"))
        .expect("raw module witness");
    assert!(module.contains("import * as ns from \"./trap-is-undefined-target-is-proxy.js\";"));
    assert!(module.contains("var nsTarget = new Proxy(ns, {});"));
    assert!(module.contains("assert(Reflect.preventExtensions(nsProxy));"));

    assert!(!TEST262_SOURCE.contains("rewrite_proxy_prevent_extensions_case"));
    assert!(!TEST262_SOURCE
        .contains("built-ins/Proxy/preventExtensions/trap-is-undefined-target-is-proxy.js"));
    assert!(CONTRACT.contains("12 physical files and 23"));
    assert!(CONTRACT.contains("one physical file and one Module"));
}

#[test]
fn source_free_consumer_covers_recursive_and_typed_handler_boundaries() {
    for marker in [
        "var deepProxy = deepTarget;",
        "deepProxy = new Proxy(deepProxy, {});",
        "deepProxy = new Proxy(deepProxy, { preventExtensions: null });",
        "deepProxy = new Proxy(deepProxy, { preventExtensions: undefined });",
        "function functionHandler() {}",
        "var arrayHandler = [];",
        "var argumentsHandler = (function() { return arguments; })(1, 2);",
        "var proxyHandler = new Proxy(proxyHandlerTarget, {",
        "var callableProxyTrap = new Proxy(function(target) {",
        "observedLookupError !== lookupSentinel",
        "observedCallError !== callSentinel",
        "Reflect.preventExtensions(falseProxy) !== false",
        "Object.preventExtensions(falseProxy)",
        "Proxy.revocable({}, {})",
    ] {
        assert!(FIXTURE.contains(marker), "missing fixture marker: {marker}");
    }
    assert!(FIXTURE.matches("deepProxy = new Proxy(deepProxy,").count() > 4);
    assert!(
        FIXTURE.contains("getterThis !== handler || trapThis !== handler || trapTarget !== target")
    );
    assert!(CLI_REGISTRATION
        .contains("fn run_wasm_backend_succeeds_for_object_prevent_extensions_proxy_fixture()"));
    assert!(CLI_REGISTRATION.contains("wasm_object_prevent_extensions_proxy.js"));
}

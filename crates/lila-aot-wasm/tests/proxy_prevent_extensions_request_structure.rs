const RUNTIME_HELPERS_SOURCE: &str = include_str!("../src/runtime_helpers.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const OBJECT_BUILTIN_SOURCE: &str = include_str!("../src/builtins/object.rs");
const REFLECT_BUILTIN_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const TEST262_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_object_prevent_extensions_proxy.js");
const CLI_REGISTRATION: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/proxy-prevent-extensions-request.md");

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../test262/vendor/test262/test/", $path)),
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
    assert!(earlier < later, "`{earlier}` must precede `{later}`");
}

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

#[test]
fn helper_catalog_and_typed_request_replace_the_fixed_depth_entry() {
    assert!(RUNTIME_HELPERS_SOURCE.contains("ObjectPreventExtensions"));
    assert!(RUNTIME_HELPERS_SOURCE.contains("Self::ObjectPreventExtensions"));
    assert!(RUNTIME_HELPERS_SOURCE
        .contains("Self::ObjectPreventExtensions => \"object_prevent_extensions\""));
    assert_before(
        RUNTIME_HELPERS_SOURCE,
        "Self::ObjectIsExtensible,",
        "Self::ObjectPreventExtensions,",
    );
    assert_before(
        RUNTIME_HELPERS_SOURCE,
        "Self::ObjectPreventExtensions,",
        "Self::ObjectReadProxy,",
    );

    for marker in [
        "struct PreventExtensionsTraversalTargetLocals",
        "struct PreventExtensionsResultLocal",
        "struct ObjectPreventExtensionsRequest",
        "struct PendingProxyPreventExtensionsTrapResultLocals",
        "struct NormalProxyPreventExtensionsTrapResultLocals",
        "compile_object_prevent_extensions_helper",
        "emit_call_object_prevent_extensions_helper",
        "object_prevent_extensions_helper_function_index",
    ] {
        assert!(
            EMIT_SOURCE.contains(marker) || OBJECTS_SOURCE.contains(marker),
            "missing typed helper marker: {marker}"
        );
    }

    let request = bounded(
        OBJECTS_SOURCE,
        "/// A complete request for the shared Proxy-aware `[[PreventExtensions]]` walk.",
        "impl ObjectPreventExtensionsRequest",
    );
    assert!(request.contains(
        "#[must_use = \"an object PreventExtensions request must be consumed by its emitter\"]"
    ));
    assert!(request.contains("target: PreventExtensionsTraversalTargetLocals"));
    assert!(request.contains("result: PreventExtensionsResultLocal"));
    assert!(!request.contains("#[derive"));

    for lifecycle_type in [
        "PreventExtensionsTraversalTargetLocals",
        "PreventExtensionsResultLocal",
        "ObjectPreventExtensionsRequest",
        "PendingProxyPreventExtensionsTrapResultLocals",
        "NormalProxyPreventExtensionsTrapResultLocals",
    ] {
        let declaration = format!("struct {lifecycle_type}");
        let declaration_prefix = OBJECTS_SOURCE
            .split_once(&declaration)
            .unwrap_or_else(|| panic!("missing lifecycle type `{lifecycle_type}`"))
            .0
            .rsplit("\n\n")
            .next()
            .unwrap_or_default();
        assert!(
            !declaration_prefix.contains("#[derive"),
            "{lifecycle_type} derives an incidental capability"
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
            assert!(
                !OBJECTS_SOURCE.contains(&format!("impl {capability} for {lifecycle_type}")),
                "{lifecycle_type} manually implements {capability}"
            );
        }
    }

    let transition = bounded(
        OBJECTS_SOURCE,
        "fn emit_normal_proxy_prevent_extensions_trap_result(",
        "fn emit_proxy_prevent_extensions_trap_result(",
    );
    assert!(transition.contains("pending: PendingProxyPreventExtensionsTrapResultLocals"));
    assert_before(
        transition,
        "self.emit_propagate_throw_from_locals_if_needed(",
        "Ok(NormalProxyPreventExtensionsTrapResultLocals(pending.0))",
    );

    let recursive_helper_call = bounded(
        OBJECTS_SOURCE,
        "fn emit_call_object_prevent_extensions_helper(",
        "pub(crate) fn emit_object_get_prototype_of_without_proxy(",
    );
    assert_before(
        recursive_helper_call,
        "self.store_call_results(helper_payload_local, helper_tag_local, function);",
        "self.emit_propagate_throw_from_locals_if_needed(",
    );
    assert_before(
        recursive_helper_call,
        "self.emit_propagate_throw_from_locals_if_needed(",
        "Instruction::LocalSet(result_local)",
    );

    let normal_consumer = bounded(
        OBJECTS_SOURCE,
        "fn emit_proxy_prevent_extensions_trap_result(",
        "pub(crate) fn emit_object_prevent_extensions(",
    );
    assert!(normal_consumer.contains("trap_result: NormalProxyPreventExtensionsTrapResultLocals"));
    assert_before(
        normal_consumer,
        "self.compile_truthy_tagged_i32(",
        "self.emit_call_object_is_extensible_helper(",
    );
    assert_before(
        normal_consumer,
        "self.emit_call_object_is_extensible_helper(",
        "Instruction::I64Const(1)",
    );

    let traversal = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_object_prevent_extensions(",
        "pub(crate) fn emit_object_is_extensible_i32(",
    );
    for marker in [
        "self.emit_load_live_proxy_slots(",
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_propagate_throw_from_locals_if_needed(",
        "self.emit_function_or_proxy_call_leave_throw_completion(",
        "PendingProxyPreventExtensionsTrapResultLocals::new(",
        "self.emit_normal_proxy_prevent_extensions_trap_result(",
        "self.emit_proxy_prevent_extensions_trap_result(",
        "self.emit_call_object_prevent_extensions_helper(",
    ] {
        assert!(
            traversal.contains(marker),
            "missing traversal marker: {marker}"
        );
    }
    assert_before(
        traversal,
        "self.emit_load_live_proxy_slots(",
        "self.emit_object_read_without_throw_propagation(",
    );
    assert_before(
        traversal,
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_propagate_throw_from_locals_if_needed(",
    );
    assert_before(
        traversal,
        "self.emit_propagate_throw_from_locals_if_needed(",
        "self.emit_is_callable_i32(",
    );
    assert_before(
        traversal,
        "self.emit_function_or_proxy_call_leave_throw_completion(",
        "PendingProxyPreventExtensionsTrapResultLocals::new(",
    );
    assert!(traversal.contains(
        "self.emit_normal_proxy_prevent_extensions_trap_result(\n            PendingProxyPreventExtensionsTrapResultLocals::new("
    ));
    assert_before(
        traversal,
        "self.emit_normal_proxy_prevent_extensions_trap_result(",
        "self.emit_proxy_prevent_extensions_trap_result(",
    );
    let nullish_fallback = traversal
        .split_once("Instruction::I32Or")
        .expect("nullish trap classification")
        .1;
    assert!(nullish_fallback.contains("self.emit_call_object_prevent_extensions_helper("));

    assert!(!OBJECTS_SOURCE.contains("emit_object_prevent_extensions_i32_with_depth"));
    assert_eq!(
        OBJECT_BUILTIN_SOURCE
            .matches("ObjectPreventExtensionsRequest::new(")
            .count(),
        3
    );
    let object_entry = bounded(
        OBJECT_BUILTIN_SOURCE,
        "pub(super) fn compile_object_prevent_extensions_builtin(",
        "pub(super) fn compile_object_prototype_proto_getter_builtin(",
    );
    assert!(object_entry.contains("ObjectPreventExtensionsRequest::new("));
    assert!(!object_entry.contains("proxy_handled_local"));
    assert!(!object_entry.contains("proxy_handler_payload_local"));

    let reflect_entry = bounded(
        REFLECT_BUILTIN_SOURCE,
        "pub(crate) fn compile_reflect_prevent_extensions_builtin(",
        "pub(crate) fn compile_reflect_is_extensible_builtin(",
    );
    assert!(reflect_entry.contains("ObjectPreventExtensionsRequest::new("));
    assert!(!reflect_entry.contains("emit_object_prevent_extensions_i32("));
}

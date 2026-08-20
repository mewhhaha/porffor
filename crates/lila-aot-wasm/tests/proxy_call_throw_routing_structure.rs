const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

#[test]
fn proxy_call_throw_routing_is_one_private_closed_domain() {
    assert!(FUNCTIONS_SOURCE.contains("\nenum ProxyCallThrowRouting {"));
    assert!(!FUNCTIONS_SOURCE.contains("pub(crate) enum ProxyCallThrowRouting"));
    assert!(!FUNCTIONS_SOURCE.contains("pub(super) enum ProxyCallThrowRouting"));

    let declaration = between(
        FUNCTIONS_SOURCE,
        "enum ProxyCallThrowRouting {",
        "}\n\nimpl ProxyCallThrowRouting",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        ["ReturnCurrentFunction,", "LeaveInCompletion,"],
        "the proxy-call throw domain must contain exactly the two current policies",
    );

    let implementation = between(
        FUNCTIONS_SOURCE,
        "impl ProxyCallThrowRouting {",
        "\n}\n\n/// A Wasm local proven",
    );
    assert!(implementation.contains("match self {"));
    assert!(implementation.contains("Self::ReturnCurrentFunction => true,"));
    assert!(implementation.contains("Self::LeaveInCompletion => false,"));
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
}

#[test]
fn only_named_wrappers_can_select_the_raw_proxy_call_route() {
    let raw_call = "self.emit_function_or_proxy_call_with_argv_inner(";
    assert_eq!(FUNCTIONS_SOURCE.matches(raw_call).count(), 2);
    assert!(!EMIT_SOURCE.contains(raw_call));
    assert!(FUNCTIONS_SOURCE.contains("\n    fn emit_function_or_proxy_call_with_argv_inner("));
    assert!(
        !FUNCTIONS_SOURCE.contains("pub(crate) fn emit_function_or_proxy_call_with_argv_inner(")
    );

    let returning_wrapper = between(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_function_or_proxy_call_with_argv_without_throw_propagation(",
        "pub(crate) fn emit_function_or_proxy_call_with_argv_leave_throw_completion(",
    );
    assert_eq!(returning_wrapper.matches(raw_call).count(), 1);
    assert_eq!(
        returning_wrapper
            .matches("ProxyCallThrowRouting::ReturnCurrentFunction")
            .count(),
        1,
    );
    assert!(!returning_wrapper.contains("ProxyCallThrowRouting::LeaveInCompletion"));

    let leaving_wrapper = between(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_function_or_proxy_call_with_argv_leave_throw_completion(",
        "fn emit_function_or_proxy_call_with_argv_inner(",
    );
    assert_eq!(leaving_wrapper.matches(raw_call).count(), 1);
    assert_eq!(
        leaving_wrapper
            .matches("ProxyCallThrowRouting::LeaveInCompletion")
            .count(),
        1,
    );
    assert!(!leaving_wrapper.contains("ProxyCallThrowRouting::ReturnCurrentFunction"));
}

#[test]
fn raw_dispatch_and_outlined_helper_preserve_the_reviewed_policy_split() {
    let raw_dispatch = between(
        FUNCTIONS_SOURCE,
        "fn emit_function_or_proxy_call_with_argv_inner(",
        "pub(crate) fn emit_function_handle_call_with_argv_inner(",
    );
    assert!(raw_dispatch.contains("throw_routing: ProxyCallThrowRouting,"));
    assert!(!raw_dispatch.contains("return_on_throw"));
    assert_eq!(
        raw_dispatch
            .matches("throw_routing.returns_current_function()")
            .count(),
        9,
        "every existing proxy-dispatch throw exit must read the typed policy",
    );

    let outlined_helper = between(
        EMIT_SOURCE,
        "fn compile_proxy_call_helper(",
        "fn compile_proxy_construct_helper(",
    );
    assert_eq!(
        outlined_helper
            .matches("self.emit_function_or_proxy_call_with_argv_leave_throw_completion(")
            .count(),
        1,
    );
    assert!(!outlined_helper.contains("emit_function_or_proxy_call_with_argv_inner"));
}

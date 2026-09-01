const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");

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
fn primitive_to_number_throw_routing_is_one_private_closed_domain() {
    assert!(OPERATIONS_SOURCE.contains("\nenum PrimitiveToNumberThrowRouting {"));
    assert!(!OPERATIONS_SOURCE.contains("pub(crate) enum PrimitiveToNumberThrowRouting"));
    assert!(!OPERATIONS_SOURCE.contains("pub(super) enum PrimitiveToNumberThrowRouting"));

    let preceding_declaration_line = OPERATIONS_SOURCE
        .split_once("enum PrimitiveToNumberThrowRouting {")
        .expect("missing PrimitiveToNumberThrowRouting declaration")
        .0
        .rsplit_once('\n')
        .map_or("", |(_, line)| line);
    assert!(!preceding_declaration_line
        .trim_start()
        .starts_with("#[derive("));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!OPERATIONS_SOURCE.contains(&format!(
            "impl {capability} for PrimitiveToNumberThrowRouting"
        )));
    }

    let declaration = between(
        OPERATIONS_SOURCE,
        "enum PrimitiveToNumberThrowRouting {",
        "}\n\nimpl PrimitiveToNumberThrowRouting",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(',') && !line.starts_with("///"))
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        ["ReturnCurrentFunction,", "LeaveInCompletion,"],
        "primitive ToNumber must expose exactly its two current throw owners",
    );

    let implementation = between(
        OPERATIONS_SOURCE,
        "impl PrimitiveToNumberThrowRouting {",
        "\n}\n\n/// The realm that owns",
    );
    assert!(implementation
        .contains("fn emit(&self, builder: &mut FunctionBuilder<'_>, function: &mut Function)"));
    assert!(implementation.contains("match self {"));
    assert!(implementation.contains(
        "Self::ReturnCurrentFunction => builder.emit_return_current_completion(function),"
    ));
    assert!(implementation.contains("Self::LeaveInCompletion => {}"));
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
}

#[test]
fn named_wrappers_are_the_only_raw_primitive_to_number_callers() {
    let raw_call = "self.emit_primitive_to_number_payload_inner(";
    assert_eq!(OPERATIONS_SOURCE.matches(raw_call).count(), 2);
    assert!(
        OPERATIONS_SOURCE.contains("\n    fn emit_primitive_to_number_payload_inner("),
        "the policy-selecting emitter must remain private",
    );
    assert!(!OPERATIONS_SOURCE.contains("pub(crate) fn emit_primitive_to_number_payload_inner("));

    let returning_wrapper = between(
        OPERATIONS_SOURCE,
        "pub(crate) fn emit_primitive_to_number_payload(",
        "pub(crate) fn emit_primitive_to_number_payload_without_throw_return(",
    );
    assert_eq!(returning_wrapper.matches(raw_call).count(), 1);
    assert_eq!(
        returning_wrapper
            .matches("PrimitiveToNumberThrowRouting::ReturnCurrentFunction")
            .count(),
        1,
    );
    assert!(!returning_wrapper.contains("PrimitiveToNumberThrowRouting::LeaveInCompletion"));

    let leaving_wrapper = between(
        OPERATIONS_SOURCE,
        "pub(crate) fn emit_primitive_to_number_payload_without_throw_return(",
        "fn emit_primitive_to_number_payload_inner(",
    );
    assert_eq!(leaving_wrapper.matches(raw_call).count(), 1);
    assert_eq!(
        leaving_wrapper
            .matches("PrimitiveToNumberThrowRouting::LeaveInCompletion")
            .count(),
        1,
    );
    assert!(!leaving_wrapper.contains("PrimitiveToNumberThrowRouting::ReturnCurrentFunction"));
}

#[test]
fn raw_emitter_preserves_both_throw_sites_and_instruction_order() {
    let raw_emitter = between(
        OPERATIONS_SOURCE,
        "fn emit_primitive_to_number_payload_inner(",
        "pub(crate) fn emit_value_to_number_payload_allow_bigint(",
    );
    assert!(raw_emitter.contains("throw_routing: PrimitiveToNumberThrowRouting,"));
    assert!(!raw_emitter.contains("return_on_throw"));
    assert_eq!(
        raw_emitter
            .matches("throw_routing.emit(self, function);")
            .count(),
        2,
        "BigInt and Symbol must both consume the typed throw policy",
    );

    for message in [
        "Cannot convert BigInt to number",
        "Cannot convert Symbol to number",
    ] {
        let throw = raw_emitter
            .find(message)
            .unwrap_or_else(|| panic!("missing numeric conversion throw: {message}"));
        let route = raw_emitter[throw..]
            .find("throw_routing.emit(self, function);")
            .map(|offset| throw + offset)
            .unwrap_or_else(|| panic!("missing typed route after: {message}"));
        let placeholder = raw_emitter[route..]
            .find("self.emit_nan_payload(function);")
            .map(|offset| route + offset)
            .unwrap_or_else(|| panic!("missing placeholder NaN after: {message}"));
        assert!(throw < route && route < placeholder);
    }
}

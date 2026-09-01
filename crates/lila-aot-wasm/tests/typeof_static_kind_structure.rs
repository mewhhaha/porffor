const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/typeof-static-kind-domain.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn static_typeof_owns_the_complete_value_kind_domain() {
    let body = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn compile_typeof_payload(",
        "pub(crate) fn emit_typeof_payload_from_tag_payload_local(",
    );
    for kind in [
        "Undefined",
        "Null",
        "Object",
        "Array",
        "Arguments",
        "Boolean",
        "Number",
        "BigInt",
        "Symbol",
        "String",
        "Dynamic",
    ] {
        assert_eq!(body.matches(&format!("ValueKind::{kind}")).count(), 1);
    }
    assert_eq!(body.matches("ValueKind::Function").count(), 2);
    assert!(body.contains("ValueKind::Object | ValueKind::Dynamic => None"));
    assert!(!body.contains("unreachable!"));
    assert!(!body.contains("_ =>"));
    assert!(!OPERATIONS_SOURCE.contains("emit_typeof_payload_for_kind"));
}

#[test]
fn static_typeof_results_and_runtime_fallback_remain_exact() {
    let body = normalized(bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn compile_typeof_payload(",
        "pub(crate) fn emit_typeof_payload_from_tag_payload_local(",
    ));
    for result in [
        "ValueKind::Undefined=>Some(\"undefined\")",
        "ValueKind::Null|ValueKind::Array|ValueKind::Arguments=>Some(\"object\")",
        "ValueKind::Boolean=>Some(\"boolean\")",
        "ValueKind::Number=>Some(\"number\")",
        "ValueKind::BigInt=>Some(\"bigint\")",
        "ValueKind::Symbol=>Some(\"symbol\")",
        "ValueKind::String=>Some(\"string\")",
        "ValueKind::Object|ValueKind::Dynamic=>None",
    ] {
        assert!(
            body.contains(result),
            "missing static typeof result: {result}"
        );
    }
    let function_kind = body
        .find("ValueKind::Function=>{")
        .expect("missing Function arm");
    let html_dda = body
        .find("self.emit_is_htmldda_function_i32(")
        .expect("missing HTMLDDA observation");
    let function_return = body[html_dda..]
        .find("returnOk(());")
        .map(|position| position + html_dda)
        .expect("missing Function early return");
    let runtime_fallback = body
        .rfind("self.compile_expr_to_locals(")
        .expect("missing runtime tag fallback");
    assert!(function_kind < html_dda);
    assert!(html_dda < function_return);
    assert!(function_return < runtime_fallback);
}

#[test]
fn contract_and_task_record_total_static_typeof_ownership() {
    for source in [CONTRACT, TASK] {
        assert!(source.contains("ValueKind"));
        assert!(source.contains("Object"));
        assert!(source.contains("Dynamic"));
        assert!(source.contains("HTMLDDA"));
    }
}

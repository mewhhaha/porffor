use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_keyed_element_projection.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn rust_sources(dir: &Path) -> String {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            if path.is_dir() {
                return rust_sources(&path);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return String::new();
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        })
        .collect()
}

#[test]
fn keyed_element_projection_is_one_private_closed_domain() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_keyed_element_projection;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_keyed_element_projection;"));
    assert!(!PROMISE_SOURCE.contains("promise_keyed_element_projection::"));
    assert!(!PROMISE_SOURCE.contains("PromiseKeyedElementProjection"));
    assert!(PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE.lines().count() <= 250);

    let declaration = bounded(
        PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE,
        "enum PromiseKeyedElementProjection {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert_eq!(declaration.matches("FulfilledValue,").count(), 1);
    assert_eq!(
        declaration
            .matches("SettlementRecord(PromiseSettlement),")
            .count(),
        1
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| line.ends_with(','))
            .count(),
        2,
        "the keyed-element projection domain must contain exactly two variants"
    );
    assert!(!declaration.contains("pub"));
    assert!(!declaration.contains("bool"));
    assert!(!PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE.contains("#[derive"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let all_sources = rust_sources(&source_root);
    assert_eq!(
        all_sources.matches("PromiseKeyedElementProjection").count(),
        6,
        "one declaration, two producers, one owned parameter and two exhaustive arms must be complete"
    );
    assert!(!all_sources.contains("promise_keyed_element_projection::"));
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!all_sources.contains(&format!(
            "impl {capability} for PromiseKeyedElementProjection"
        )));
    }
}

#[test]
fn named_wrappers_own_the_projection_choice() {
    let fulfilled_value = bounded(
        PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE,
        "pub(crate) fn emit_promise_all_keyed_resolve_element(",
        "pub(crate) fn emit_promise_all_settled_keyed_element(",
    );
    assert_eq!(
        fulfilled_value
            .matches("PromiseKeyedElementProjection::FulfilledValue")
            .count(),
        1
    );
    assert!(!fulfilled_value.contains("PromiseSettlement"));
    assert!(!fulfilled_value.contains("true"));
    assert!(!fulfilled_value.contains("false"));

    let settlement_record = bounded(
        PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE,
        "pub(crate) fn emit_promise_all_settled_keyed_element(",
        "fn emit_promise_all_keyed_element(",
    );
    assert!(settlement_record.contains("settlement: PromiseSettlement,"));
    assert_eq!(
        settlement_record
            .matches("PromiseKeyedElementProjection::SettlementRecord(settlement)")
            .count(),
        1
    );
    assert!(!settlement_record.contains("true"));
    assert!(!settlement_record.contains("false"));
}

#[test]
fn keyed_element_helper_exhaustively_projects_the_stored_value() {
    let signature = bounded(
        PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE,
        "fn emit_promise_all_keyed_element(",
        ") -> Result<(), EmitError> {",
    );
    assert!(signature.contains("projection: PromiseKeyedElementProjection,"));
    assert!(!signature.contains("settled_record"));
    assert!(!signature.contains("settlement: PromiseSettlement"));
    assert!(!signature.contains("bool"));

    let helper = bounded(
        PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE,
        "fn emit_promise_all_keyed_element(",
        "\n}",
    );
    let projection = bounded(
        helper,
        "match projection {",
        "self.emit_object_define_enumerable_data(",
    );
    assert_eq!(
        projection
            .matches("PromiseKeyedElementProjection::FulfilledValue => {}")
            .count(),
        1
    );
    assert_eq!(
        projection
            .matches("PromiseKeyedElementProjection::SettlementRecord(settlement) => {")
            .count(),
        1
    );
    assert_eq!(
        projection
            .matches("self.emit_self_backed_promise_settlement_record_allocation_context(")
            .count(),
        1
    );
    assert_eq!(
        projection
            .matches("self.emit_alloc_promise_settlement_record(")
            .count(),
        1
    );
    for arm in [
        "PromiseSettlement::Fulfill => (\"fulfilled\", \"value\")",
        "PromiseSettlement::Reject => (\"rejected\", \"reason\")",
    ] {
        assert_eq!(projection.matches(arm).count(), 1, "settlement arm `{arm}`");
    }
    assert!(!projection.contains("_ =>"));
    assert!(!projection.contains("unreachable!"));

    for retired in [
        "settled_record: bool",
        "emit_promise_all_keyed_element(false",
        "emit_promise_all_keyed_element(true",
    ] {
        assert!(
            !PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE.contains(retired),
            "retired `{retired}`"
        );
        assert!(!PROMISE_SOURCE.contains(retired), "retired `{retired}`");
    }
}

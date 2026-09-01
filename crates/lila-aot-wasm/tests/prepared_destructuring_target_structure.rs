use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const ARRAY_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_destructuring_iterators.js");
const PRIVATE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_private_destructuring_reference_order.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/prepared-destructuring-target.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn prepared_target_is_one_private_must_use_capability_free_domain() {
    let declaration = bounded(
        CONTROL_FLOW_SOURCE,
        "#[must_use = \"a prepared destructuring target must be consumed by its write\"]",
        "#[must_use = \"a prepared destructuring property key must be consumed by its write\"]",
    );
    assert!(declaration.contains("enum PreparedDestructuringTarget<'a> {"));
    assert!(!declaration.contains("#[derive("));
    for variant in [
        "Binding {",
        "AssignmentIdentifier(",
        "Property {",
        "Private {",
        "NestedArray(",
        "NestedObject(",
    ] {
        assert_eq!(
            declaration.matches(variant).count(),
            1,
            "variant `{variant}`"
        );
    }
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
        assert!(!CONTROL_FLOW_SOURCE.contains(&format!(
            "impl {capability} for PreparedDestructuringTarget"
        )));
    }
}

#[test]
fn preparation_exhaustively_constructs_the_matching_target_variant() {
    let preparation = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn prepare_destructuring_target<'b>(",
        "    fn put_destructuring_target(",
    );
    assert!(preparation.contains("match target {"));
    assert!(!preparation.contains(".clone()"));
    for forbidden in ["if let DestructuringTargetIr", "_ =>", "unreachable!"] {
        assert!(!preparation.contains(forbidden), "found `{forbidden}`");
    }

    for variant in [
        "Binding",
        "AssignmentIdentifier",
        "AssignmentProperty",
        "AssignmentPrivate",
        "NestedArray",
        "NestedObject",
    ] {
        assert_eq!(
            preparation
                .matches(&format!("DestructuringTargetIr::{variant}"))
                .count(),
            1,
            "IR target `{variant}`"
        );
    }
    for variant in [
        "Binding",
        "AssignmentIdentifier",
        "Property",
        "Private",
        "NestedArray",
        "NestedObject",
    ] {
        assert_eq!(
            preparation
                .matches(&format!("PreparedDestructuringTarget::{variant}"))
                .count(),
            1,
            "prepared target `{variant}`"
        );
    }
}

#[test]
fn write_consumes_only_the_prepared_target_without_a_parallel_ir_discriminant() {
    let write = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn put_destructuring_target(",
        "    pub(crate) fn emit_iterator_close_condition_i32(",
    );
    let signature = bounded(write, "&mut self,", ") -> Result<(), EmitError> {");
    assert!(signature.contains("prepared: PreparedDestructuringTarget<'_>,"));
    assert!(!signature.contains("target: &DestructuringTargetIr"));
    assert!(write.contains("match prepared {"));
    assert!(!write.contains("DestructuringTargetIr::"));
    assert!(!write.contains("unreachable!"));
    assert!(!write.contains("_ =>"));

    for variant in [
        "Binding",
        "AssignmentIdentifier",
        "Property",
        "Private",
        "NestedArray",
        "NestedObject",
    ] {
        assert_eq!(
            write
                .matches(&format!("PreparedDestructuringTarget::{variant}"))
                .count(),
            1,
            "consumed target `{variant}`"
        );
    }
    assert_eq!(
        write
            .matches("match key {\n                    PreparedDestructuringPropertyKey::Static(_)")
            .count(),
        1
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("put_destructuring_target(")
            .count(),
        5
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("prepare_destructuring_target(")
            .count(),
        4
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "enum PreparedDestructuringTarget"),
        1
    );
}

#[test]
fn focused_evidence_covers_direct_property_nested_and_private_writes() {
    for marker in [
        "[first = 4, second = 5, ...rest] = exhaustedIterable",
        "([orderedTarget()[orderedKey()]] = orderedSource())",
        "let [[nestedValue = 13] = []] = [[]]",
        "[...restTarget[restKey]] = [14, 15]",
    ] {
        assert!(ARRAY_FIXTURE.contains(marker), "array fixture `{marker}`");
    }
    assert!(PRIVATE_FIXTURE.contains("({ value: this.#value } = source)"));

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("PreparedDestructuringTarget"));
        assert!(evidence.contains("must-use"));
        assert!(evidence.contains("six-variant"));
        assert!(without_whitespace(evidence).contains("parallelIRdiscriminant"));
        assert!(evidence.contains("Batch AD"));
    }
}

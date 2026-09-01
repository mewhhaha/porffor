const OPERATION_SOURCE: &str =
    include_str!("../src/builtins/string/string_well_formed_operation.rs");
const STRING_SOURCE: &str = include_str!("../src/builtins/string.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn well_formed_operation_is_a_private_capability_free_domain() {
    let declaration_start = OPERATION_SOURCE
        .find("enum StringWellFormedOperation {")
        .expect("missing well-formed operation declaration");
    let preceding_declaration = OPERATION_SOURCE[..declaration_start]
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .expect("missing declaration before well-formed operation");
    assert_eq!(preceding_declaration.trim(), "use super::*;");

    let declaration = bounded(
        OPERATION_SOURCE,
        "enum StringWellFormedOperation {",
        "\n}\n\nimpl FunctionBuilder<'_> {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Check,", "Repair,"]);

    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(
            !OPERATION_SOURCE.contains(&format!("impl {capability} for StringWellFormedOperation"))
        );
    }
    assert!(!OPERATION_SOURCE.contains("derive("));
    assert!(!STANDARD_SOURCE.contains("StringWellFormedOperation"));
}

#[test]
fn named_builtin_entry_points_are_the_only_operation_producers() {
    let wrappers = normalized(bounded(
        OPERATION_SOURCE,
        "impl FunctionBuilder<'_> {",
        "\n}\n\nfn emit(",
    ));
    assert!(wrappers.contains("fnemit_string_is_well_formed_builtin("));
    assert!(wrappers.contains("emit(self,StringWellFormedOperation::Check,function)"));
    assert!(wrappers.contains("fnemit_string_to_well_formed_builtin("));
    assert!(wrappers.contains("emit(self,StringWellFormedOperation::Repair,function)"));

    assert_eq!(
        OPERATION_SOURCE
            .matches("StringWellFormedOperation::Check")
            .count(),
        2
    );
    assert_eq!(
        OPERATION_SOURCE
            .matches("StringWellFormedOperation::Repair")
            .count(),
        2
    );
}

#[test]
fn consuming_match_owns_algorithm_and_result_tag_together() {
    let projection = normalized(bounded(
        OPERATION_SOURCE,
        "match operation {",
        "\n    }\n    function.instruction(&Instruction::End);",
    ));
    let check = bounded(
        &projection,
        "StringWellFormedOperation::Check=>{",
        "}StringWellFormedOperation::Repair=>{",
    );
    let repair = bounded(&projection, "StringWellFormedOperation::Repair=>{", "}");

    assert!(check.contains("emit_string_is_well_formed_payload_from_local"));
    assert!(check.contains("ValueKind::Boolean.tag()"));
    assert!(!check.contains("ValueKind::String.tag()"));
    assert!(repair.contains("emit_string_to_well_formed_payload_from_local"));
    assert!(repair.contains("ValueKind::String.tag()"));
    assert!(!repair.contains("ValueKind::Boolean.tag()"));
    assert!(!projection.contains("_=>"));
    assert!(!projection.contains("unreachable!"));
}

#[test]
fn standard_dispatch_cannot_supply_a_mode_or_retag_the_result() {
    let is_well_formed = normalized(bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::StringPrototypeIsWellFormed => {",
        "StandardBuiltinId::StringPrototypeToWellFormed => {",
    ));
    assert_eq!(
        is_well_formed,
        "self.emit_string_is_well_formed_builtin(function)?;}"
    );

    let to_well_formed = normalized(bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::StringPrototypeToWellFormed => {",
        "StandardBuiltinId::StringPrototypeTrim",
    ));
    assert_eq!(
        to_well_formed,
        "self.emit_string_to_well_formed_builtin(function)?;}"
    );

    for raw_emitter in [
        "emit_string_is_well_formed_payload_from_local",
        "emit_string_to_well_formed_payload_from_local",
    ] {
        assert_eq!(STANDARD_SOURCE.matches(raw_emitter).count(), 0);
        assert_eq!(
            STRING_SOURCE.matches(&format!("fn {raw_emitter}(")).count(),
            1
        );
        assert_eq!(
            STRING_SOURCE
                .matches(&format!("pub(crate) fn {raw_emitter}("))
                .count(),
            0
        );
    }
}

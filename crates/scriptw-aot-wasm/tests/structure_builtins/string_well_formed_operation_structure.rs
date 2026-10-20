const OPERATION_SOURCE: &str =
    include_str!("../../src/builtins/string/string_well_formed_operation.rs");
const STRING_SOURCE: &str = include_str!("../../src/builtins/string.rs");
const STANDARD_SOURCE: &str = include_str!("../../src/builtins/standard.rs");

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
    assert_eq!(variants, ["IsWellFormed,", "ToWellFormed,"]);

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
        "    fn emit_native_string_well_formed(",
    ));
    assert!(wrappers.contains("fnemit_string_is_well_formed_builtin("));
    assert!(wrappers.contains(
        "self.emit_native_string_well_formed(StringWellFormedOperation::IsWellFormed,f)"
    ));
    assert!(wrappers.contains("fnemit_string_to_well_formed_builtin("));
    assert!(wrappers.contains(
        "self.emit_native_string_well_formed(StringWellFormedOperation::ToWellFormed,f)"
    ));

    assert_eq!(
        OPERATION_SOURCE
            .matches("StringWellFormedOperation::IsWellFormed")
            .count(),
        3
    );
    assert_eq!(
        OPERATION_SOURCE
            .matches("StringWellFormedOperation::ToWellFormed")
            .count(),
        3
    );
}

#[test]
fn consuming_match_owns_algorithm_and_result_tag_together() {
    let allocation = normalized(bounded(
        OPERATION_SOURCE,
        "let construction = match &operation {",
        "valid.store(f);",
    ));
    assert!(allocation.contains("StringWellFormedOperation::IsWellFormed=>None,"));
    assert!(allocation
        .contains("StringWellFormedOperation::ToWellFormed=>Some(StringConstruction::allocate("));
    assert!(!allocation.contains("_=>"));

    let projection = normalized(bounded(
        OPERATION_SOURCE,
        "match operation {",
        "output.set_normal(&result, f);",
    ));
    let check = bounded(
        &projection,
        "StringWellFormedOperation::IsWellFormed=>",
        "StringWellFormedOperation::ToWellFormed=>{",
    );
    let repair = bounded(
        &projection,
        "StringWellFormedOperation::ToWellFormed=>{",
        "repaired.clear(f);",
    );
    assert_eq!(check, "result.set_boolean(valid,f),");
    assert!(repair
        .contains("construction.expect(\"ToWellFormedownsitscompleteconstruction\").publish(s,f)"));
    assert!(repair.contains("result.set_reference(&repaired,s,f)"));
    assert!(!repair.contains("set_boolean"));
    assert!(!projection.contains("_=>"));
    assert!(!projection.contains("unreachable!"));
    assert_eq!(OPERATION_SOURCE.matches("match &operation {").count(), 1);
    assert_eq!(OPERATION_SOURCE.matches("match operation {").count(), 1);
    assert!(OPERATION_SOURCE
        .contains("self.emit_with_native_string_receiver(f, |b, string, output, _, f|"));
    assert!(OPERATION_SOURCE.contains("b.emit_gc_string_code_unit_i32(string, index, f)"));
    assert!(
        OPERATION_SOURCE.find("match operation {").unwrap()
            < OPERATION_SOURCE
                .find("output.set_normal(&result, f)")
                .unwrap()
    );
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
        "self.emit_string_is_well_formed_builtin(function)?}"
    );

    let to_well_formed = normalized(bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::StringPrototypeToWellFormed => {",
        "StandardBuiltinId::StringPrototypeTrim",
    ));
    assert_eq!(
        to_well_formed,
        "self.emit_string_to_well_formed_builtin(function)?}"
    );

    let helper = "emit_native_string_well_formed(";
    assert_eq!(
        OPERATION_SOURCE.matches(helper).count(),
        3,
        "one private typed consumer and two named producers"
    );
    assert_eq!(
        OPERATION_SOURCE
            .matches("    fn emit_native_string_well_formed(")
            .count(),
        1
    );
    assert!(!OPERATION_SOURCE.contains("pub(crate) fn emit_native_string_well_formed("));
    assert!(!STRING_SOURCE.contains(helper));
    assert!(!STANDARD_SOURCE.contains(helper));
}

use std::fs;
use std::path::Path;

const BOOLEAN_SOURCE: &str = include_str!("../src/builtins/boolean.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/boolean-prototype-operation.md");
const T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const T24: &str = include_str!("../../../tasks/24-globals-errors-annexb-host.md");

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
fn boolean_domains_have_exact_rows_without_derived_policy_capabilities() {
    let declarations = normalized(bounded(
        BOOLEAN_SOURCE,
        "use super::super::*;",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    assert_eq!(
        declarations,
        concat!(
            "enumBooleanBuiltin{Constructor,PrototypeToString,PrototypeValueOf,}",
            "enumBooleanPrototypeOperation{ToString,ValueOf,}"
        )
    );

    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!BOOLEAN_SOURCE.contains(&format!("{capability} for BooleanPrototypeOperation")));
        assert!(!BOOLEAN_SOURCE.contains(&format!("{capability} for BooleanBuiltin")));
    }

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&src, "BooleanPrototypeOperation"), 6);
    assert_eq!(count_in_rust_sources(&src, "BooleanBuiltin"), 8);

    for evidence in [CONTRACT, T02, T24] {
        assert!(evidence.contains("private `BooleanBuiltin`"));
        assert!(evidence.contains("fixed Boolean entries"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new Boolean behavior"));
    }
}

#[test]
fn outer_dispatch_has_two_named_prototype_operation_forwarders() {
    let dispatch = normalized(bounded(
        BOOLEAN_SOURCE,
        "fn emit_boolean_builtin(",
        "fn emit_boolean_prototype_builtin(",
    ));
    for mapping in [
        "BooleanBuiltin::PrototypeToString=>{self.emit_boolean_prototype_builtin(BooleanPrototypeOperation::ToString,function)?}",
        "BooleanBuiltin::PrototypeValueOf=>{self.emit_boolean_prototype_builtin(BooleanPrototypeOperation::ValueOf,function)?}",
    ] {
        assert_eq!(dispatch.matches(mapping).count(), 1, "mapping `{mapping}`");
    }
    assert_eq!(
        dispatch.matches("BooleanBuiltin::Constructor=>{").count(),
        1
    );
    assert!(!dispatch.contains("PrototypeToString|BooleanBuiltin::PrototypeValueOf"));

    assert!(!STANDARD_SOURCE.contains("BooleanBuiltin"));
    assert!(!STANDARD_SOURCE.contains("emit_boolean_builtin("));
    for (standard_builtin, entry, variant) in [
        (
            "BooleanConstructor",
            "emit_boolean_constructor_builtin",
            "Constructor",
        ),
        (
            "BooleanPrototypeToString",
            "emit_boolean_prototype_to_string_builtin",
            "PrototypeToString",
        ),
        (
            "BooleanPrototypeValueOf",
            "emit_boolean_prototype_value_of_builtin",
            "PrototypeValueOf",
        ),
    ] {
        assert_eq!(
            STANDARD_SOURCE
                .matches(&format!("StandardBuiltinId::{standard_builtin} =>"))
                .count(),
            1,
            "standard route `{standard_builtin}`"
        );
        assert_eq!(
            STANDARD_SOURCE
                .matches(&format!("self.{entry}(function)?"))
                .count(),
            1,
            "standard entry `{entry}`"
        );
        assert_eq!(
            BOOLEAN_SOURCE
                .matches(&format!(
                    "self.emit_boolean_builtin(BooleanBuiltin::{variant}, function)"
                ))
                .count(),
            1,
            "fixed producer `{variant}`"
        );
    }
}

#[test]
fn shared_receiver_validation_precedes_the_result_operation() {
    let helper = bounded(
        BOOLEAN_SOURCE,
        "fn emit_boolean_prototype_builtin(",
        "\n    }\n}",
    );
    assert!(helper.contains("operation: BooleanPrototypeOperation,"));
    assert_eq!(
        helper
            .matches("Boolean.prototype method requires a Boolean receiver")
            .count(),
        2
    );
    assert_eq!(
        helper
            .matches("self.emit_return_current_completion(function);")
            .count(),
        2
    );
    let primitive_check = helper
        .find("ValueKind::Boolean.tag() as i64")
        .expect("primitive Boolean receiver check");
    let object_check = helper
        .find("ValueKind::Object.tag() as i64")
        .expect("boxed Boolean object check");
    let boxed_kind_check = helper
        .find("BOXED_PRIMITIVE_KIND_BOOLEAN as i64")
        .expect("boxed Boolean kind check");
    let result_operation = helper
        .find("match operation {")
        .expect("result operation match");
    assert!(primitive_check < object_check);
    assert!(object_check < boxed_kind_check);
    assert!(boxed_kind_check < result_operation);
}

#[test]
fn result_operation_is_one_exact_exhaustive_two_arm_match() {
    let helper = bounded(
        BOOLEAN_SOURCE,
        "fn emit_boolean_prototype_builtin(",
        "\n    }\n}",
    );
    let result_operation = normalized(bounded(
        helper,
        "match operation {",
        "self.release_temp_local(boolean_payload_local);",
    ));
    assert_eq!(
        result_operation,
        concat!(
            "BooleanPrototypeOperation::ValueOf=>{",
            "function.instruction(&Instruction::LocalGet(boolean_payload_local));",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag()asi64));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));}",
            "BooleanPrototypeOperation::ToString=>{",
            "function.instruction(&Instruction::LocalGet(boolean_payload_local));",
            "function.instruction(&Instruction::I64Eqz);",
            "function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));",
            "function.instruction(&Instruction::I64Const(self.strings.payload(\"false\")));",
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::I64Const(self.strings.payload(\"true\")));",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(ValueKind::String.tag()asi64));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));}}"
        )
    );
    for forbidden in [
        "_=>",
        "ifoperation",
        "operation==",
        "operation!=",
        "matches!",
        "unreachable!",
    ] {
        assert!(!result_operation.contains(forbidden));
    }
}

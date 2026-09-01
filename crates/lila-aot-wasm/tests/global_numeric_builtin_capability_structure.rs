use std::fs;
use std::path::Path;

const GLOBAL_NUMERIC_SOURCE: &str = include_str!("../src/builtins/global_numeric.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/global-numeric-builtin-capability.md");
const TASK: &str = include_str!("../../../tasks/24-globals-errors-annexb-host.md");

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
fn global_numeric_builtin_is_the_exact_non_capability_domain() {
    let declaration = normalized(bounded(
        GLOBAL_NUMERIC_SOURCE,
        "use super::super::*;",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    assert_eq!(declaration, "enumGlobalNumericBuiltin{IsFinite,IsNaN,}");

    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!GLOBAL_NUMERIC_SOURCE.contains(&format!("{capability} for GlobalNumericBuiltin")));
    }

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&src, "GlobalNumericBuiltin"), 8);
    assert_eq!(
        count_in_rust_sources(&src, "GlobalNumericBuiltin::IsFinite"),
        3
    );
    assert_eq!(
        count_in_rust_sources(&src, "GlobalNumericBuiltin::IsNaN"),
        3
    );
}

#[test]
fn standard_dispatch_uses_two_fixed_global_predicate_entries() {
    let producers = normalized(bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::GlobalIsFinite =>",
        "StandardBuiltinId::MathAbs =>",
    ));
    assert_eq!(
        producers,
        concat!(
            "self.emit_global_is_finite_builtin(function)?,",
            "StandardBuiltinId::GlobalIsNaN=>",
            "self.emit_global_is_nan_builtin(function)?,"
        )
    );
    assert!(!STANDARD_SOURCE.contains("GlobalNumericBuiltin"));

    let fixed_entries = normalized(bounded(
        GLOBAL_NUMERIC_SOURCE,
        "    pub(super) fn emit_global_is_finite_builtin(",
        "\n}",
    ));
    assert_eq!(
        fixed_entries
            .matches("self.emit_global_numeric_builtin(")
            .count(),
        2
    );
    assert_eq!(
        fixed_entries
            .matches("GlobalNumericBuiltin::IsFinite")
            .count(),
        1
    );
    assert_eq!(
        fixed_entries.matches("GlobalNumericBuiltin::IsNaN").count(),
        1
    );
}

#[test]
fn exhaustive_emitter_preserves_both_result_policies_and_common_order() {
    let emitter = normalized(bounded(
        GLOBAL_NUMERIC_SOURCE,
        "    fn emit_global_numeric_builtin(",
        "    pub(super) fn emit_global_is_finite_builtin(",
    ));
    assert_eq!(
        emitter,
        concat!(
            "&mutself,builtin:GlobalNumericBuiltin,function:&mutFunction,",
            ")->Result<(),EmitError>{",
            "matchbuiltin{",
            "GlobalNumericBuiltin::IsFinite|GlobalNumericBuiltin::IsNaN=>{",
            "letarg_payload_local=self.reserve_temp_local();",
            "letarg_tag_local=self.reserve_temp_local();",
            "self.emit_builtin_arg_to_locals(0,arg_payload_local,arg_tag_local,function);",
            "self.emit_value_to_number_payload(arg_tag_local,arg_payload_local,function)?;",
            "function.instruction(&Instruction::LocalSet(arg_payload_local));",
            "self.emit_return_current_completion_if_throw(function);",
            "function.instruction(&Instruction::LocalGet(arg_payload_local));",
            "function.instruction(&Instruction::F64ReinterpretI64);",
            "function.instruction(&Instruction::LocalGet(arg_payload_local));",
            "function.instruction(&Instruction::F64ReinterpretI64);",
            "function.instruction(&Instruction::F64Ne);",
            "matchbuiltin{",
            "GlobalNumericBuiltin::IsFinite=>{",
            "function.instruction(&Instruction::I32Eqz);",
            "forinfinitein[f64::INFINITY,f64::NEG_INFINITY]{",
            "function.instruction(&Instruction::LocalGet(arg_payload_local));",
            "function.instruction(&Instruction::F64ReinterpretI64);",
            "function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));",
            "function.instruction(&Instruction::F64Ne);",
            "function.instruction(&Instruction::I32And);}}",
            "GlobalNumericBuiltin::IsNaN=>{}}",
            "function.instruction(&Instruction::I64ExtendI32U);",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag()asi64));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "self.release_temp_local(arg_tag_local);",
            "self.release_temp_local(arg_payload_local);}}",
            "Ok(())}"
        )
    );
    for forbidden in ["_=>", "==", "!=", "matches!(", "unreachable!"] {
        assert!(!emitter.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn contract_and_t24_record_the_source_equivalent_capability_closure() {
    for marker in [
        "private, non-derived `GlobalNumericBuiltin::{IsFinite, IsNaN}`",
        "fixed producer entries",
        "does not claim the full",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("global-numeric-builtin-capability.md"));
}

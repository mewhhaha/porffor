use std::fs;
use std::path::Path;

const MATH_SOURCE: &str = include_str!("../src/builtins/math.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
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
fn math_extremum_is_the_exact_private_capability_free_domain() {
    let declaration = bounded(
        MATH_SOURCE,
        "enum MathExtremum {",
        "const MATH_SUM_PRECISE_MAX_COUNT",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take_while(|line| *line != "}")
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Minimum,", "Maximum,"]);
    for forbidden in ["pub", "#[derive"] {
        assert!(!declaration.contains(forbidden), "found `{forbidden}`");
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
        assert!(!MATH_SOURCE.contains(&format!("impl {capability} for MathExtremum")));
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "MathExtremum"),
        5,
        "the declaration, impl, typed consumer and two producers must own every mention"
    );
}

#[test]
fn math_extremum_projects_both_identity_and_instruction_exhaustively() {
    let implementation = bounded(
        MATH_SOURCE,
        "impl MathExtremum {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert_eq!(implementation.matches("match self {").count(), 2);
    assert_eq!(implementation.matches("Self::Minimum =>").count(), 2);
    assert_eq!(implementation.matches("Self::Maximum =>").count(), 2);
    for forbidden in ["_ =>", "matches!", "==", "!="] {
        assert!(!implementation.contains(forbidden), "found `{forbidden}`");
    }

    let identity = normalized(bounded(
        implementation,
        "    const fn identity(&self) -> f64 {",
        "    fn emit_combine(",
    ));
    assert_eq!(
        identity,
        "matchself{Self::Minimum=>f64::INFINITY,Self::Maximum=>f64::NEG_INFINITY,}}"
    );

    let combine = normalized(bounded(implementation, "    fn emit_combine(", "\n    }"));
    assert!(combine.starts_with("&self,"));
    assert_eq!(combine.matches("matchself{").count(), 1);
    assert!(combine.contains(
        "matchself{Self::Minimum=>function.instruction(&Instruction::F64Min),Self::Maximum=>function.instruction(&Instruction::F64Max),};"
    ));
    assert_before(
        &combine,
        "Instruction::LocalGet(accumulator_local)",
        "matchself{",
    );
    assert_before(
        &combine,
        "matchself{",
        "Instruction::LocalSet(accumulator_local)",
    );
}

#[test]
fn math_extremum_producers_and_reduction_order_are_exact() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_math_extremum_builtin("),
        3,
        "the definition and two Math builtin arms must be the full call census"
    );
    for (builtin, variant) in [("Min", "Minimum"), ("Max", "Maximum")] {
        assert_eq!(
            count_in_rust_sources(&source_root, &format!("MathExtremum::{variant}")),
            1,
            "Math.{builtin} must be the only `{variant}` producer"
        );
    }

    let producers = normalized(MATH_SOURCE);
    for mapping in [
        "MathBuiltin::Min=>self.emit_math_extremum_builtin(MathExtremum::Minimum,arg_payload_local,arg_tag_local,function,)?",
        "MathBuiltin::Max=>self.emit_math_extremum_builtin(MathExtremum::Maximum,arg_payload_local,arg_tag_local,function,)?",
    ] {
        assert_eq!(producers.matches(mapping).count(), 1, "producer `{mapping}`");
    }

    let emitter = bounded(
        MATH_SOURCE,
        "    fn emit_math_extremum_builtin(",
        "    pub(super) fn emit_math_abs_builtin(",
    );
    assert!(emitter.contains("extremum: MathExtremum,"));
    assert!(!emitter.contains("extremum.clone()"));
    assert_eq!(emitter.matches("reserve_temp_local()").count(), 1);
    assert_eq!(
        emitter
            .matches("release_temp_local(argument_index_local)")
            .count(),
        1
    );
    assert_eq!(emitter.matches("emit_array_read(").count(), 1);
    assert_eq!(emitter.matches("emit_value_to_number_payload(").count(), 1);
    assert_eq!(
        emitter
            .matches("emit_return_current_completion_if_throw(function)")
            .count(),
        1
    );
    assert_eq!(emitter.matches("extremum.emit_combine(").count(), 1);
    assert_eq!(
        emitter
            .matches("Instruction::Block(BlockType::Empty)")
            .count(),
        1
    );
    assert_eq!(
        emitter
            .matches("Instruction::Loop(BlockType::Empty)")
            .count(),
        1
    );
    assert_eq!(emitter.matches("Instruction::BrIf(1)").count(), 1);
    assert_eq!(emitter.matches("Instruction::Br(0)").count(), 1);
    assert_eq!(
        emitter
            .matches("Instruction::LocalGet(argument_index_local)")
            .count(),
        2
    );
    assert_eq!(
        emitter
            .matches("Instruction::LocalSet(argument_index_local)")
            .count(),
        2
    );
    assert_eq!(emitter.matches("Instruction::I64Const(1)").count(), 1);
    assert_eq!(emitter.matches("Instruction::I64Add").count(), 1);
    assert_eq!(emitter.matches("Instruction::End").count(), 2);
    assert_eq!(
        emitter
            .matches("Instruction::LocalSet(arg_payload_local)")
            .count(),
        1
    );

    assert_before(emitter, "extremum.identity()", "Instruction::Loop");
    assert_before(emitter, "Instruction::BrIf(1)", "self.emit_array_read(");
    assert_before(
        emitter,
        "self.emit_array_read(",
        "self.emit_value_to_number_payload(",
    );
    assert_before(
        emitter,
        "self.emit_value_to_number_payload(",
        "Instruction::LocalSet(arg_payload_local)",
    );
    assert_before(
        emitter,
        "Instruction::LocalSet(arg_payload_local)",
        "self.emit_return_current_completion_if_throw(function)",
    );
    assert_before(
        emitter,
        "self.emit_return_current_completion_if_throw(function)",
        "extremum.emit_combine(",
    );
    assert_before(
        emitter,
        "extremum.emit_combine(",
        "Instruction::I64Const(1)",
    );
    assert_before(emitter, "Instruction::Br(0)", "Instruction::End");
    assert_before(
        emitter,
        "Instruction::End",
        "self.release_temp_local(argument_index_local)",
    );
    let normalized_emitter = normalized(emitter);
    assert!(normalized_emitter.contains(
        "Instruction::Br(0));function.instruction(&Instruction::End);function.instruction(&Instruction::End);self.release_temp_local(argument_index_local);Ok(())"
    ));
}

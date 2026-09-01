use std::fs;
use std::path::Path;

const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
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
fn number_conversion_accepts_only_the_existing_arithmetic_operator_domain() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for retired_pattern in [
        "enum NumericBinaryOperator",
        "NumericBinaryOperator::",
        ": NumericBinaryOperator",
        "use crate::operations::NumericBinaryOperator",
    ] {
        assert_eq!(count_in_rust_sources(&source_root, retired_pattern), 0);
    }
    assert!(!EXPRESSIONS_SOURCE.contains("use crate::operations::NumericBinaryOperator"));

    let signature = concat!(
        "    pub(crate) fn compile_operand_pair_to_number_locals(\n",
        "        &mut self,\n",
        "        operator: ArithmeticBinaryOp,\n",
    );
    assert_eq!(OPERATIONS_SOURCE.matches(signature).count(), 1);
    assert!(!OPERATIONS_SOURCE.contains("operator: bool"));
}

#[test]
fn arithmetic_operator_exhaustively_owns_the_conversion_order() {
    let selector = bounded(
        OPERATIONS_SOURCE,
        "fn arithmetic_applies_to_primitive_before_numeric(",
        "/// The complete runtime result domain of `ToNumeric` for unary bitwise",
    );
    let selector_code = selector
        .lines()
        .filter(|line| !line.trim_start().starts_with("///"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        normalized(&selector_code),
        concat!(
            "operator:ArithmeticBinaryOp)->bool{matchoperator{",
            "ArithmeticBinaryOp::Add=>true,",
            "ArithmeticBinaryOp::Sub|ArithmeticBinaryOp::Mul|",
            "ArithmeticBinaryOp::Div|ArithmeticBinaryOp::Mod|",
            "ArithmeticBinaryOp::Exp=>false,}}",
        )
    );
    assert!(!selector.contains("_ =>"));
    assert!(!selector.contains("#[derive"));
}

#[test]
fn every_number_pair_caller_forwards_its_arithmetic_operator() {
    assert_eq!(
        EXPRESSIONS_SOURCE
            .matches("self.compile_operand_pair_to_number_locals(")
            .count(),
        3
    );
    assert_eq!(
        normalized(EXPRESSIONS_SOURCE)
            .matches("self.compile_operand_pair_to_number_locals(*op,lhs,rhs,")
            .count(),
        3
    );
}

#[test]
fn shared_number_pair_body_preserves_evaluation_and_conversion_order() {
    let body = normalized(bounded(
        OPERATIONS_SOURCE,
        "        let lhs_payload = self.reserve_temp_local();",
        "    /// ToNumeric on one already evaluated operand, into `number_local`.",
    ));
    let ordered_steps = [
        "self.compile_expr_to_locals(lhs,lhs_payload,lhs_tag,function)?;",
        "self.compile_expr_to_locals(rhs,rhs_payload,rhs_tag,function)?;",
        "ifarithmetic_applies_to_primitive_before_numeric(operator){",
        "self.emit_to_primitive_from_raw_locals(ToPrimitiveHint::Number,lhs_payload,lhs_tag,lhs_primitive_payload,lhs_primitive_tag,function,)?;",
        "self.emit_to_primitive_from_raw_locals(ToPrimitiveHint::Number,rhs_payload,rhs_tag,rhs_primitive_payload,rhs_primitive_tag,function,)?;",
        "self.emit_operand_to_number_local(lhs_primitive_payload,lhs_primitive_tag,lhs_number_local,function,)?;",
        "self.emit_operand_to_number_local(rhs_primitive_payload,rhs_primitive_tag,rhs_number_local,function,)?;",
    ];
    let mut previous = 0;
    for step in ordered_steps {
        let position = body
            .find(step)
            .unwrap_or_else(|| panic!("missing ordered step: {step}"));
        assert!(position >= previous, "step out of order: {step}");
        previous = position;
    }
}

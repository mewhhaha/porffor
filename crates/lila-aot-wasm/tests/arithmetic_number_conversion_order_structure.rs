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
fn numeric_conversion_uses_the_existing_arithmetic_domain_without_number_only_routes() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for retired_pattern in [
        "enum NumericBinaryOperator",
        "NumericBinaryOperator::",
        ": NumericBinaryOperator",
        "compile_operand_pair_to_number_locals",
        "emit_operand_to_number_local",
        "arithmetic_applies_to_primitive_before_numeric",
    ] {
        assert_eq!(count_in_rust_sources(&source_root, retired_pattern), 0);
    }
    assert!(normalized(OPERATIONS_SOURCE).contains(concat!(
        "pub(crate)fncompile_coercive_binary_number_to_locals(",
        "&mutself,op:ArithmeticBinaryOp,"
    )));
}

#[test]
fn both_coercive_expression_consumers_use_the_tagged_arithmetic_owner() {
    let source = normalized(EXPRESSIONS_SOURCE);
    assert_eq!(
        source
            .matches("ExprIr::CoerciveBinaryNumber{op,lhs,rhs}=>{")
            .count(),
        2
    );
    for output in [
        "self.scratch_local,self.result_tag_local",
        "payload_local,tag_local",
    ] {
        let route = format!(
            "ExprIr::CoerciveBinaryNumber{{op,lhs,rhs}}=>{{self.compile_coercive_binary_number_to_locals(*op,lhs,rhs,{output},function,)?;"
        );
        assert_eq!(source.matches(&route).count(), 1, "{output}");
    }
}

#[test]
fn addition_delegates_before_reserving_numeric_operands() {
    let body = normalized(bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn compile_coercive_binary_number_to_locals(",
        "    pub(crate) fn emit_primitive_to_numeric_locals_without_throw_return(",
    ));
    assert!(body.contains(concat!(
        ")->Result<(),EmitError>{ifmatches!(op,ArithmeticBinaryOp::Add){",
        "returnself.compile_coercive_add_to_locals(lhs,rhs,payload_local,tag_local,function,);}",
        "letlhs_payload_local=self.reserve_temp_local();"
    )));
    let addition = normalized(bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn compile_coercive_add_to_locals(",
        "    pub(crate) fn compile_expr_to_object_locals(",
    ));
    let primitives = addition
        .find(concat!(
            "self.compile_operand_pair_to_primitive_locals(lhs,rhs,ToPrimitiveHint::Default,",
            "lhs_payload,lhs_tag,rhs_payload,rhs_tag,function,)?;"
        ))
        .expect("addition converts both primitives with the default hint");
    let string_choice = addition
        .find("ValueKind::String.tag()")
        .expect("string dispatch");
    let numeric = addition
        .find("self.emit_primitive_to_numeric_locals_without_throw_return(")
        .expect("numeric conversion after the string branch");
    assert!(primitives < string_choice && string_choice < numeric);
}

#[test]
fn numeric_arithmetic_evaluates_both_operands_before_ordered_conversion_and_type_check() {
    let body = normalized(bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn compile_coercive_binary_number_to_locals(",
        "    pub(crate) fn emit_primitive_to_numeric_locals_without_throw_return(",
    ));
    let ordered_steps = [
        "self.compile_expr_to_locals(lhs,lhs_payload_local,lhs_tag_local,function)?;",
        "self.compile_expr_to_locals(rhs,rhs_payload_local,rhs_tag_local,function)?;",
        "self.emit_value_to_numeric_locals(lhs_payload_local,lhs_tag_local,function)?;",
        "self.emit_value_to_numeric_locals(rhs_payload_local,rhs_tag_local,function)?;",
        "self.emit_is_bigint_tag_i32(lhs_tag_local,function);",
        "self.emit_is_bigint_tag_i32(rhs_tag_local,function);",
        "function.instruction(&Instruction::I32Ne);",
    ];
    let mut remaining = body.as_str();
    for step in ordered_steps {
        remaining = remaining
            .split_once(step)
            .unwrap_or_else(|| panic!("missing ordered step: {step}"))
            .1;
    }
}

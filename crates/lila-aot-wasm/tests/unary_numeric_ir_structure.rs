use std::fs;
use std::path::Path;

const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const OPERATION_SOURCE: &str = include_str!("../../lila-ir/src/operations.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const EXPRESSION_SOURCE: &str = include_str!("../src/expressions.rs");
const WASM_OPERATION_SOURCE: &str = include_str!("../src/operations.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const CLI_NUMERIC_TESTS: &str = include_str!("../../lila-cli/tests/cli/language_numerics.rs");
const CLI_BITWISE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_bigint_bitwise_core.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/numeric-conversion-codomains.md");
const TASK: &str = include_str!("../../../tasks/20-number-bigint-math-json.md");

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
fn unary_numeric_kind_is_the_exact_private_no_capability_domain() {
    let preceding_item = concat!(
        "fn arithmetic_applies_to_primitive_before_numeric(operator: ArithmeticBinaryOp) -> bool {\n",
        "    match operator {\n",
        "        ArithmeticBinaryOp::Add => true,\n",
        "        ArithmeticBinaryOp::Sub\n",
        "        | ArithmeticBinaryOp::Mul\n",
        "        | ArithmeticBinaryOp::Div\n",
        "        | ArithmeticBinaryOp::Mod\n",
        "        | ArithmeticBinaryOp::Exp => false,\n",
        "    }\n",
        "}\n\n",
    );
    assert_eq!(WASM_OPERATION_SOURCE.matches(preceding_item).count(), 1);
    let declaration_region = bounded(
        WASM_OPERATION_SOURCE,
        preceding_item,
        "/// Which realm environment an outlined numeric-conversion helper may receive.",
    );
    assert!(!declaration_region.contains("#["));
    let declaration_code = declaration_region
        .lines()
        .filter(|line| !line.trim_start().starts_with("///"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        normalized(&declaration_code),
        "enumUnaryNumericKind{Number,BigInt,}"
    );
    assert!(!declaration_region.contains("pub enum UnaryNumericKind"));
    assert!(!declaration_region.contains("pub(crate) enum UnaryNumericKind"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "UnaryNumericKind"),
        6,
        "the declaration, typed parameter, two producers and two exhaustive arms own every mention"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "UnaryNumericKind::Number"),
        2
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "UnaryNumericKind::BigInt"),
        2
    );
    assert!(!WASM_OPERATION_SOURCE.contains("impl UnaryNumericKind"));
    assert!(!WASM_OPERATION_SOURCE.contains("for UnaryNumericKind"));
}

#[test]
fn unary_bitwise_dispatch_preserves_both_producers_and_branch_order() {
    let dispatch = normalized(bounded(
        WASM_OPERATION_SOURCE,
        "    pub(crate) fn compile_unary_bitwise_numeric_to_locals(",
        "    #[allow(clippy::too_many_arguments)]",
    ));
    assert_eq!(
        dispatch,
        concat!(
            "&mutself,op:UnaryBitwiseOp,operand:&TypedExpr,payload_local:u32,",
            "tag_local:u32,function:&mutFunction,)->Result<(),EmitError>{",
            "letoperand_payload_local=self.reserve_temp_local();",
            "letoperand_tag_local=self.reserve_temp_local();",
            "self.compile_expr_to_locals(operand,operand_payload_local,operand_tag_local,function)?;",
            "self.emit_value_to_numeric_locals(operand_payload_local,operand_tag_local,function)?;",
            "self.emit_is_bigint_tag_i32(operand_tag_local,function);",
            "self.open_frame(ControlFrameKind::If,function);",
            "self.emit_unary_numeric_kind_to_locals(UnaryNumericKind::BigInt,op,",
            "operand_payload_local,operand_tag_local,payload_local,tag_local,function,)?;",
            "self.pop_control(ControlFrameKind::If);",
            "function.instruction(&Instruction::Else);",
            "self.emit_unary_numeric_kind_to_locals(UnaryNumericKind::Number,op,",
            "operand_payload_local,operand_tag_local,payload_local,tag_local,function,)?;",
            "function.instruction(&Instruction::End);",
            "self.release_temp_local(operand_tag_local);",
            "self.release_temp_local(operand_payload_local);Ok(())}"
        )
    );
}

#[test]
fn unary_numeric_kind_exhaustively_emits_bigint_and_number_complement() {
    let consumer = normalized(bounded(
        WASM_OPERATION_SOURCE,
        "    fn emit_unary_numeric_kind_to_locals(",
        "    /// Emits ECMA-262 ToUint32 steps 2-5 for a Number payload.",
    ));
    assert_eq!(
        consumer,
        concat!(
            "&mutself,kind:UnaryNumericKind,op:UnaryBitwiseOp,",
            "operand_payload_local:u32,operand_tag_local:u32,payload_local:u32,",
            "tag_local:u32,function:&mutFunction,)->Result<(),EmitError>{",
            "match(kind,op){(UnaryNumericKind::BigInt,UnaryBitwiseOp::Complement)=>self.",
            "emit_bigint_complement_to_locals(operand_payload_local,operand_tag_local,",
            "payload_local,tag_local,function,),",
            "(UnaryNumericKind::Number,UnaryBitwiseOp::Complement)=>{",
            "self.emit_to_uint32_i64_from_number_payload(operand_payload_local,",
            "operand_payload_local,function,);",
            "function.instruction(&Instruction::LocalGet(operand_payload_local));",
            "function.instruction(&Instruction::I32WrapI64);",
            "function.instruction(&Instruction::I32Const(-1));",
            "function.instruction(&Instruction::I32Xor);",
            "function.instruction(&Instruction::I64ExtendI32S);",
            "function.instruction(&Instruction::F64ConvertI64S);",
            "function.instruction(&Instruction::I64ReinterpretF64);",
            "function.instruction(&Instruction::LocalSet(payload_local));",
            "function.instruction(&Instruction::I64Const(ValueKind::Number.tag()asi64));",
            "function.instruction(&Instruction::LocalSet(tag_local));Ok(())}}}"
        )
    );
    for forbidden in ["_=>", "unreachable!", "kind==", "kind!=", "matches!(kind"] {
        assert!(!consumer.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn contract_and_behavior_witness_cover_both_unary_numeric_kinds() {
    assert!(CONTRACT.contains("UnaryNumericKind"));
    assert!(CONTRACT.contains("cargo test -p lila-aot-wasm --test unary_numeric_ir_structure"));
    assert!(TASK.contains("UnaryNumericKind"));
    assert!(CLI_NUMERIC_TESTS.contains("fn run_wasm_backend_succeeds_for_bigint_bitwise_fixture()"));
    for marker in [
        "~0n",
        "~(-a)",
        "~0",
        "~Infinity",
        "complementTrace",
        "throwingComplement",
    ] {
        assert!(
            CLI_BITWISE_FIXTURE.contains(marker),
            "missing unary complement witness `{marker}`"
        );
    }
}

#[test]
fn unary_plus_and_minus_have_distinct_ir_states() {
    let unary_plus = bounded(IR_SOURCE, "    UnaryPlus {", "    UnaryMinusNumeric {");
    let unary_minus = bounded(
        IR_SOURCE,
        "    UnaryMinusNumeric {",
        "    UnaryBitwiseNumeric {",
    );
    assert_eq!(unary_plus.trim(), "expr: Box<TypedExpr>,\n    },");
    assert_eq!(unary_minus.trim(), "expr: Box<TypedExpr>,\n    },");
    assert!(!IR_SOURCE.contains("UnaryNumber"));
    assert!(!IR_SOURCE.contains("UnaryBigInt"));
    assert!(!OPERATION_SOURCE.contains("UnaryNumericOp"));
}

#[test]
fn lowering_keeps_to_number_and_to_numeric_domains_separate() {
    let unary_lowering = bounded(
        LOWERING_SOURCE,
        "            UnaryOp::Plus => {",
        "            UnaryOp::Not =>",
    );
    let (plus, minus) = unary_lowering
        .split_once("UnaryOp::Minus => {")
        .expect("unary-minus lowering");
    assert_eq!(plus.matches("ExprIr::UnaryPlus").count(), 1);
    assert!(plus.contains("static_to_number_expr"));
    assert!(!plus.contains("numeric_domain"));

    assert_eq!(minus.matches("ExprIr::UnaryMinusNumeric").count(), 1);
    assert_eq!(
        minus.matches("numeric_domain(primitive.as_ref())").count(),
        1
    );
    assert!(minus.contains("ValueKind::Number"));
    assert!(minus.contains("ValueKind::BigInt"));
    assert!(!minus.contains("ExprIr::UnaryPlus"));
}

#[test]
fn wasm_minus_dispatches_exhaustively_after_to_numeric() {
    let payload_dispatch = bounded(
        EXPRESSION_SOURCE,
        "            ExprIr::UnaryPlus { expr } => {",
        "            ExprIr::UnaryBitwiseNumeric { op, expr } =>",
    );
    assert_eq!(
        payload_dispatch
            .matches("compile_expr_to_number_payload")
            .count(),
        1
    );
    assert_eq!(
        payload_dispatch
            .matches("compile_unary_minus_numeric_to_locals")
            .count(),
        1
    );

    let minus_emitter = bounded(
        WASM_OPERATION_SOURCE,
        "    pub(crate) fn compile_unary_minus_numeric_to_locals(",
        "    /// Evaluates one unary-bitwise operand",
    );
    assert_eq!(
        minus_emitter
            .matches("emit_value_to_numeric_locals")
            .count(),
        1
    );
    assert_eq!(minus_emitter.matches("emit_is_bigint_tag_i32").count(), 1);
    assert_eq!(minus_emitter.matches("BigIntHelperOp::Negate").count(), 1);
    assert_eq!(minus_emitter.matches("Instruction::F64Neg").count(), 1);
    assert!(!minus_emitter.contains("emit_value_to_number_payload"));

    let dynamic_tag_projection = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn expr_result_tag_is_runtime_dynamic(expr: &ExprIr) -> bool {",
        "pub(crate) fn count_param_locals",
    );
    assert!(dynamic_tag_projection.contains("ExprIr::UnaryMinusNumeric { .. }"));
}

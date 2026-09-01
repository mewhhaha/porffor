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
fn limb_operation_is_the_exact_private_capability_free_domain() {
    let declaration_marker = "enum MathSumPreciseLimbOperation {";
    let declaration_offset = MATH_SOURCE
        .find(declaration_marker)
        .expect("limb operation declaration");
    let preceding_item_end = MATH_SOURCE[..declaration_offset]
        .rfind('}')
        .expect("item before limb operation declaration");
    let following_item_offset = MATH_SOURCE[declaration_offset..]
        .find("struct MathSumPreciseAccumulator")
        .map(|offset| declaration_offset + offset)
        .expect("item after limb operation declaration");
    assert_eq!(
        normalized(&MATH_SOURCE[preceding_item_end + 1..following_item_offset]),
        "enumMathSumPreciseLimbOperation{Add,Subtract,}",
        "the exact declaration region must remain private and attribute-free"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "MathSumPreciseLimbOperation"),
        12,
        "the declaration, typed parameter, two producers and eight exhaustive arms own every mention"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "MathSumPreciseLimbOperation::Add"),
        5,
        "one positive producer and four Add arms"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "MathSumPreciseLimbOperation::Subtract"),
        5,
        "one negative producer and four Subtract arms"
    );
    assert!(!MATH_SOURCE.contains("impl MathSumPreciseLimbOperation"));
}

#[test]
fn limb_operation_projects_arithmetic_and_carry_polarity_exhaustively() {
    let fold = normalized(bounded(
        MATH_SOURCE,
        "    fn emit_math_sum_precise_fold_limbs(",
        "    fn emit_math_sum_precise_add_finite(",
    ));
    assert!(fold.contains("operation:MathSumPreciseLimbOperation,"));
    assert_eq!(fold.matches("match&operation{").count(), 4);
    assert_eq!(
        fold.matches("MathSumPreciseLimbOperation::Add=>").count(),
        4
    );
    assert_eq!(
        fold.matches("MathSumPreciseLimbOperation::Subtract=>")
            .count(),
        4
    );

    let projections = [
        concat!(
            "Instruction::LocalGet(old_local));",
            "function.instruction(&Instruction::LocalGet(addend_local));",
            "match&operation{",
            "MathSumPreciseLimbOperation::Add=>function.instruction(&Instruction::I64Add),",
            "MathSumPreciseLimbOperation::Subtract=>function.instruction(&Instruction::I64Sub),",
            "};function.instruction(&Instruction::LocalSet(partial_local));"
        ),
        concat!(
            "Instruction::LocalGet(partial_local));",
            "function.instruction(&Instruction::LocalGet(old_local));",
            "match&operation{",
            "MathSumPreciseLimbOperation::Add=>function.instruction(&Instruction::I64LtU),",
            "MathSumPreciseLimbOperation::Subtract=>function.instruction(&Instruction::I64GtU),",
            "};function.instruction(&Instruction::I64ExtendI32U);",
            "function.instruction(&Instruction::LocalSet(next_carry_local));"
        ),
        concat!(
            "Instruction::LocalGet(partial_local));",
            "function.instruction(&Instruction::LocalGet(carry_local));",
            "match&operation{",
            "MathSumPreciseLimbOperation::Add=>function.instruction(&Instruction::I64Add),",
            "MathSumPreciseLimbOperation::Subtract=>function.instruction(&Instruction::I64Sub),",
            "};function.instruction(&Instruction::LocalSet(updated_local));"
        ),
        concat!(
            "Instruction::LocalGet(updated_local));",
            "function.instruction(&Instruction::LocalGet(partial_local));",
            "match&operation{",
            "MathSumPreciseLimbOperation::Add=>function.instruction(&Instruction::I64LtU),",
            "MathSumPreciseLimbOperation::Subtract=>function.instruction(&Instruction::I64GtU),",
            "};function.instruction(&Instruction::I64ExtendI32U);",
            "function.instruction(&Instruction::LocalGet(next_carry_local));",
            "function.instruction(&Instruction::I64Or);",
            "function.instruction(&Instruction::LocalSet(next_carry_local));"
        ),
    ];
    let mut preceding_projection_end = 0;
    for projection in projections {
        assert_eq!(
            fold.matches(projection).count(),
            1,
            "projection `{projection}`"
        );
        let projection_offset = fold.find(projection).expect("exact projection");
        assert!(
            projection_offset >= preceding_projection_end,
            "projection `{projection}` is out of arithmetic/carry order"
        );
        preceding_projection_end = projection_offset + projection.len();
    }

    for forbidden in ["_=>", "operation==", "operation!=", "matches!("] {
        assert!(!fold.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn finite_term_sign_selects_the_exact_limb_operation() {
    let finite = normalized(bounded(
        MATH_SOURCE,
        "    fn emit_math_sum_precise_add_finite(",
        "    fn emit_math_sum_precise_accept_number(",
    ));
    let producer = concat!(
        "function.instruction(&Instruction::LocalGet(number_bits_local));",
        "function.instruction(&Instruction::I64Const(63));",
        "function.instruction(&Instruction::I64ShrU);",
        "function.instruction(&Instruction::I32WrapI64);",
        "function.instruction(&Instruction::If(BlockType::Empty));",
        "self.emit_math_sum_precise_fold_limbs(accumulator,first_index_local,low_local,",
        "high_local,MathSumPreciseLimbOperation::Subtract,function,);",
        "function.instruction(&Instruction::Else);",
        "self.emit_math_sum_precise_fold_limbs(accumulator,first_index_local,low_local,",
        "high_local,MathSumPreciseLimbOperation::Add,function,);",
        "function.instruction(&Instruction::End);"
    );
    assert_eq!(finite.matches(producer).count(), 1);
    assert_eq!(
        finite.matches("emit_math_sum_precise_fold_limbs(").count(),
        2
    );
}

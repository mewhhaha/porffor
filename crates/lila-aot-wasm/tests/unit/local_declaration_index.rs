use super::*;

#[test]
fn indexed_declarations_keep_zero_runs_boundaries_and_exact_wasm_bytes() {
    let runs = [
        (0, ValType::I64),
        (2, ValType::I32),
        (0, ValType::I64),
        (1, ValType::F64),
        (3, ValType::I64),
        (0, ValType::I32),
    ];
    let declarations = LocalDeclarations::from_runs(runs);
    assert_eq!(declarations.count(), 6);
    for (index, ty) in [
        ValType::I32,
        ValType::I32,
        ValType::F64,
        ValType::I64,
        ValType::I64,
        ValType::I64,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations.type_at(index as u32), Some(ty));
    }
    assert_eq!(declarations.type_at(6), None);
    assert_eq!(declarations.type_at(u32::MAX), None);
    let mut actual = declarations.encode_body();
    let mut expected = wasm_encoder::Function::new(runs);
    actual.instruction(&Instruction::End);
    expected.instruction(&Instruction::End);
    let mut actual_bytes = Vec::new();
    let mut expected_bytes = Vec::new();
    wasm_encoder::Encode::encode(&actual, &mut actual_bytes);
    wasm_encoder::Encode::encode(&expected, &mut expected_bytes);
    assert_eq!(actual_bytes, expected_bytes);
    let mut fork = declarations.fork_for_body();
    fork.push(ValType::I64);
    assert_eq!(fork.type_at(6), Some(ValType::I64));
    assert_eq!(declarations.type_at(6), None);
}

#[test]
fn compressed_declarations_reject_overflow_before_mutation() {
    let mut declarations = LocalDeclarations::from_runs([(u32::MAX, ValType::I32)]);
    assert_eq!(declarations.type_at(u32::MAX - 1), Some(ValType::I32));
    assert_eq!(declarations.type_at(u32::MAX), None);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || declarations.push(ValType::I64)
    ))
    .is_err());
    assert_eq!(declarations.count(), u32::MAX);
    assert_eq!(declarations.runs.len(), 1);
}

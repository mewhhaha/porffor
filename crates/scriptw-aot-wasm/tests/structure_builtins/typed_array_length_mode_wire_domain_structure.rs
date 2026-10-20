fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

#[test]
fn frozen_length_mode_projection_receipt_remains_stable() {
    let legacy_rows = concat!(
        "        function.instruction(&Instruction::I64Const(0));\n",
        "        function.instruction(&Instruction::I64Const(0));\n",
        "        function.instruction(&Instruction::I64Const(0));\n",
        "        function.instruction(&Instruction::I64Const(1));\n",
        "                function.instruction(&Instruction::I64Const(0));\n",
        "                function.instruction(&Instruction::I64Const(0));\n",
    );
    let normalized = without_whitespace(legacy_rows);
    assert_eq!(
        (legacy_rows.len(), fnv1a(legacy_rows)),
        (358, 0xc988_3610_80d6_b5cc)
    );
    assert_eq!(
        (normalized.len(), fnv1a(&normalized)),
        (288, 0x7e69_1158_ebde_da94)
    );
}

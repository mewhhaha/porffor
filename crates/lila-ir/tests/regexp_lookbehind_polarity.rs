use lila_ir::{RegExpProgram, REGEXP_OPCODE_LOOKBEHIND_END, REGEXP_OPCODE_LOOKBEHIND_FAILURE};

fn polarity_bits(program: &RegExpProgram) -> (u64, u64) {
    let end = program
        .instructions
        .iter()
        .find(|instruction| instruction.opcode == REGEXP_OPCODE_LOOKBEHIND_END)
        .expect("lookbehind end instruction");
    let failure = program
        .instructions
        .iter()
        .find(|instruction| instruction.opcode == REGEXP_OPCODE_LOOKBEHIND_FAILURE)
        .expect("lookbehind failure instruction");
    (end.operand1 >> 63, failure.operand1)
}

#[test]
fn positive_and_negative_lookbehind_preserve_distinct_matcher_polarity() {
    let positive = RegExpProgram::compile("(?<=a)b", "").unwrap();
    let negative = RegExpProgram::compile("(?<!a)b", "").unwrap();

    assert_eq!(polarity_bits(&positive), (0, 0));
    assert_eq!(polarity_bits(&negative), (1, 1));
}

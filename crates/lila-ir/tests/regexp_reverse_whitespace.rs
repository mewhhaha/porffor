use lila_ir::{
    RegExpProgram, REGEXP_OPCODE_NOT_WHITESPACE, REGEXP_OPCODE_PROGRESS_CHECK,
    REGEXP_OPCODE_PROGRESS_SPLIT, REGEXP_OPCODE_WHITESPACE,
};

#[test]
fn lookbehind_admits_both_whitespace_polarities_in_every_grammar() {
    for flags in ["", "u", "v", "i", "ui", "vi"] {
        let program = RegExpProgram::compile(r"(?<=a\s\S)b", flags).unwrap();
        let whitespace = program
            .instructions
            .iter()
            .filter(|instruction| {
                matches!(
                    instruction.opcode,
                    REGEXP_OPCODE_WHITESPACE | REGEXP_OPCODE_NOT_WHITESPACE
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(whitespace.len(), 2, "{flags}");
        assert_eq!(whitespace[0].opcode, REGEXP_OPCODE_NOT_WHITESPACE);
        assert_eq!(whitespace[1].opcode, REGEXP_OPCODE_WHITESPACE);
        assert!(whitespace
            .iter()
            .all(|instruction| instruction.operand0 == 0 && instruction.operand1 == 0));
    }
}

#[test]
fn reverse_whitespace_repetitions_remain_consuming() {
    for pattern in [r"(?<=\s+)x", r"(?<=\S*)x", r"(?<=(\s|\S)+)x"] {
        let program = RegExpProgram::compile(pattern, "").unwrap();
        assert!(!program.instructions.iter().any(|instruction| matches!(
            instruction.opcode,
            REGEXP_OPCODE_PROGRESS_CHECK | REGEXP_OPCODE_PROGRESS_SPLIT
        )));
    }
}

#[test]
fn nested_assertions_keep_whitespace_admission_in_both_directions() {
    for pattern in [
        r"(?<=a(?=\s)\s)b",
        r"(?<=a\s(?<=\s))b",
        r"(?<=a(?=\S)\S)b",
        r"(?<!\s)b",
        r"(?<=\s\b)word",
    ] {
        for flags in ["", "u", "v"] {
            RegExpProgram::compile(pattern, flags).unwrap();
        }
    }
}

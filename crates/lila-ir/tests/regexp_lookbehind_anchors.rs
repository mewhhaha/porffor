use lila_ir::{RegExpProgram, REGEXP_OPCODE_ASSERT_END, REGEXP_OPCODE_ASSERT_START};

#[test]
fn lookbehind_accepts_start_and_end_assertions_in_either_polarity() {
    for (pattern, anchor) in [
        (r"(?<=^abc)def", REGEXP_OPCODE_ASSERT_START),
        (r"(?<!(^|[ab]))\w{2}", REGEXP_OPCODE_ASSERT_START),
        (r"\w+(?<=$)", REGEXP_OPCODE_ASSERT_END),
        (r"\w+(?<!$)", REGEXP_OPCODE_ASSERT_END),
    ] {
        for flags in ["", "m", "u", "mu"] {
            let program = RegExpProgram::compile(pattern, flags)
                .unwrap_or_else(|error| panic!("{pattern}/{flags}: {error}"));
            assert!(program
                .instructions
                .iter()
                .any(|instruction| instruction.opcode == anchor));
        }
    }
}

#[test]
fn lookbehind_anchor_admission_preserves_other_capability_rejections() {
    for pattern in [r"(?<=(?<=a)b)c", r"(a)(?<=\1)b"] {
        let error = RegExpProgram::compile(pattern, "").expect_err(pattern);
        assert!(
            error.to_string().contains("unsupported matcher atom"),
            "{error}"
        );
    }
}

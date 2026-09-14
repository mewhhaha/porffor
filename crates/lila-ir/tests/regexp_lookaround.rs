use lila_ir::{
    RegExpCompileErrorKind, RegExpProgram, REGEXP_OPCODE_ASSERT_END, REGEXP_OPCODE_DOT,
    REGEXP_OPCODE_LOOKAROUND_END, REGEXP_OPCODE_LOOKAROUND_START,
};

#[test]
fn lookaheads_parse_the_complete_disjunction_instead_of_a_literal_byte() {
    for (pattern, opcode) in [
        ("(?=.)", REGEXP_OPCODE_DOT),
        ("(?=$)", REGEXP_OPCODE_ASSERT_END),
    ] {
        let program = RegExpProgram::compile(pattern, "").unwrap();
        assert!(program
            .instructions
            .iter()
            .any(|instruction| instruction.opcode == opcode));
    }
    for pattern in [
        r"(?=\:)",
        "(?=ab)",
        "(?!ab)",
        "(?=)",
        "(?!)",
        "(?=(?<name>a+))a*b\\k<name>",
    ] {
        RegExpProgram::compile(pattern, "").unwrap();
    }
}

#[test]
fn nested_lookarounds_encode_each_assertion_and_its_enclosing_direction() {
    let program = RegExpProgram::compile("(?<=a(?=(?<!x)b))b", "").unwrap();
    let starts = program
        .instructions
        .iter()
        .filter(|instruction| instruction.opcode == REGEXP_OPCODE_LOOKAROUND_START)
        .map(|instruction| instruction.operand0)
        .collect::<Vec<_>>();
    let parents = program
        .instructions
        .iter()
        .filter(|instruction| instruction.opcode == REGEXP_OPCODE_LOOKAROUND_END)
        .map(|instruction| (instruction.operand1 >> 62) & 1)
        .collect::<Vec<_>>();
    assert_eq!(starts, [1, 0, 1]);
    assert_eq!(parents, [0, 1, 0]);
}

#[test]
fn quantified_assertions_respect_the_legacy_lookahead_grammar() {
    for pattern in ["(?=ab)?", "(?!ab)+", "(?=a){2,3}"] {
        RegExpProgram::compile(pattern, "").unwrap();
        for flags in ["u", "v"] {
            assert_eq!(
                RegExpProgram::compile(pattern, flags).unwrap_err().kind,
                RegExpCompileErrorKind::InvalidSyntax
            );
        }
    }
    for pattern in ["(?<=a)?", "(?<!a)+", "^*", "$?", "(?=a**)", "(?=a"] {
        for flags in ["", "u", "v"] {
            assert_eq!(
                RegExpProgram::compile(pattern, flags).unwrap_err().kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}/{flags}"
            );
        }
    }
}

#[test]
fn negative_and_optional_lookaround_captures_leave_nullable_backreferences() {
    for pattern in [r"(?!(a))\1", r"(?=(a))*\1", r"(?<!(a))\1"] {
        let program = RegExpProgram::compile(pattern, "").unwrap();
        let reference = program
            .instructions
            .iter()
            .find(|instruction| instruction.opcode == lila_ir::REGEXP_OPCODE_NUMBERED_BACKREFERENCE)
            .unwrap();
        assert_eq!(reference.operand1, 0, "{pattern}");
    }
}

#[test]
fn reverse_scalar_ranges_and_legacy_pairs_reach_existing_matcher_atoms() {
    for (pattern, flags) in [
        (r"(?<=^\uD83D)\uDE00", "m"),
        ("(?<=😀)x", ""),
        ("(?<=😀)x", "u"),
        (r"(?<=\p{Script=Han})x", "v"),
    ] {
        RegExpProgram::compile(pattern, flags).unwrap();
    }
}

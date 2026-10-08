use lila_ir::{
    RegExpCompileErrorKind, RegExpProgram, ValidatedRegExpProgram, REGEXP_OPCODE_ASSERT_START,
    REGEXP_OPCODE_CAPTURE_START, REGEXP_OPCODE_LOOKAROUND_START,
    REGEXP_OPCODE_NUMBERED_BACKREFERENCE,
};

#[test]
fn structural_empty_choices_share_the_empty_program_without_decimal_expansion() {
    let expected =
        ValidatedRegExpProgram::from_program(&RegExpProgram::compile("", "").unwrap()).unwrap();
    for pattern in [
        "|",
        "(?:|)",
        "(?:(?:|)(?:||)){18446744073709551616}",
        "(?:(?:|){18446744073709551616}){18446744073709551616}",
        "(?:(?i:|)(?-i:||)){18446744073709551616,18446744073709551618}?",
        "(?:|)*?",
    ] {
        let program = RegExpProgram::compile(pattern, "").unwrap();
        assert!(program.repeat_bounds.is_empty(), "{pattern}");
        let admitted = ValidatedRegExpProgram::from_program(&program).unwrap();
        assert_eq!(admitted.bytes(), expected.bytes(), "{pattern}");
    }
}

#[test]
fn empty_composition_preserves_enclosing_capture_assertion_and_real_alternative() {
    for (pattern, simplified) in [
        ("((?:|){18446744073709551616})", "()"),
        ("(?<value>(?:|){18446744073709551616})", "(?<value>)"),
        ("(?<=((?:|){18446744073709551616}))a", "(?<=())a"),
        ("(?:(?:|){18446744073709551616}|a)b", "(?:|a)b"),
    ] {
        let program =
            ValidatedRegExpProgram::from_program(&RegExpProgram::compile(pattern, "d").unwrap())
                .unwrap();
        let expected =
            ValidatedRegExpProgram::from_program(&RegExpProgram::compile(simplified, "d").unwrap())
                .unwrap();
        assert_eq!(program.bytes(), expected.bytes(), "{pattern}");
    }
    for (pattern, opcode) in [
        ("(?:()|){3}", REGEXP_OPCODE_CAPTURE_START),
        ("(?:^|){3}", REGEXP_OPCODE_ASSERT_START),
        ("(?:(?!)|){3}", REGEXP_OPCODE_LOOKAROUND_START),
        (r"()(?:\1|){3}", REGEXP_OPCODE_NUMBERED_BACKREFERENCE),
    ] {
        let program = RegExpProgram::compile(pattern, "").unwrap();
        assert!(
            program.instructions.iter().any(|row| row.opcode == opcode),
            "{pattern}"
        );
    }
}

#[test]
fn structural_empty_elision_does_not_hide_parser_or_capture_name_errors() {
    for pattern in [
        "(?:|){3,2}",
        "(?:(?:|){3,2}){0}",
        "(?i-:|){3,2}",
        r"(?:(?:)|\k<missing>){0}",
        "(?:(?<value>)(?<value>)){0}",
    ] {
        assert_eq!(
            RegExpProgram::compile(pattern, "u").unwrap_err().kind,
            RegExpCompileErrorKind::InvalidSyntax,
            "{pattern}",
        );
    }
}

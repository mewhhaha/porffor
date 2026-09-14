use lila_ir::{
    RegExpCompileErrorKind, RegExpProgram, REGEXP_OPCODE_LITERAL_ASCII,
    REGEXP_OPCODE_LITERAL_CODE_POINT, REGEXP_OPCODE_NEGATIVE_ASCII_CLASS,
    REGEXP_OPCODE_NUMBERED_BACKREFERENCE, REGEXP_OPCODE_POSITIVE_ASCII_CLASS,
    REGEXP_OPCODE_UNICODE_PROPERTY,
};

fn contains(pattern: &str, flags: &str, character: char) -> bool {
    let program = RegExpProgram::compile(pattern, flags).unwrap();
    let instruction = &program.instructions[0];
    let code_point = character as u32;
    match instruction.opcode {
        REGEXP_OPCODE_LITERAL_ASCII | REGEXP_OPCODE_LITERAL_CODE_POINT => {
            instruction.operand0 == u64::from(code_point)
        }
        REGEXP_OPCODE_POSITIVE_ASCII_CLASS | REGEXP_OPCODE_NEGATIVE_ASCII_CLASS => {
            let member = code_point < 128
                && if code_point < 64 {
                    instruction.operand0 & (1 << code_point) != 0
                } else {
                    instruction.operand1 & (1 << (code_point - 64)) != 0
                };
            member ^ (instruction.opcode == REGEXP_OPCODE_NEGATIVE_ASCII_CLASS)
        }
        REGEXP_OPCODE_UNICODE_PROPERTY => {
            let start = instruction.operand0 as usize;
            let end = start + (instruction.operand1 >> 1) as usize;
            program.ranges[start..end]
                .iter()
                .any(|(start, end)| (*start..=*end).contains(&code_point))
                ^ (instruction.operand1 & 1 != 0)
        }
        opcode => panic!("expected a character-set atom, got {opcode}"),
    }
}

#[test]
fn unicode_simple_folding_preserves_default_turkic_and_sharp_s_rules() {
    for flags in ["ui", "vi"] {
        for pattern in ["ß", "[ß]"] {
            assert!(contains(pattern, flags, 'ẞ'));
        }
        for pattern in ["i", "[i]", "I", "[I]"] {
            assert!(!contains(pattern, flags, 'ı'));
            assert!(!contains(pattern, flags, 'İ'));
        }
        assert!(contains("𐐀", flags, '𐐨'));
        assert!(contains(r"\u{10400}", flags, '𐐨'));
    }
}

#[test]
fn legacy_folding_uses_one_code_unit_and_rejects_expansions_and_ascii_crossings() {
    assert!(contains("é", "i", 'É'));
    assert!(contains("[σ]", "i", 'ς'));
    assert!(!contains("[ß]", "i", 'ẞ'));
    assert!(!contains("[k]", "i", 'K'));
    assert!(!contains("[s]", "i", 'ſ'));
    assert!(!contains("[i]", "i", 'ı'));
}

#[test]
fn unicode_word_and_negated_ascii_sets_close_before_complementing() {
    for flags in ["ui", "vi"] {
        for character in ['K', 'ſ'] {
            assert!(contains(r"\w", flags, character));
            assert!(!contains(r"\W", flags, character));
        }
        assert!(contains("[k]", flags, 'K'));
        assert!(!contains("[^k]", flags, 'K'));
    }
}

#[test]
fn word_escapes_keep_their_complement_inside_ordinary_and_nested_classes() {
    for flags in ["ui", "vi"] {
        for character in ['k', 'K', 's', 'S', 'K', 'ſ'] {
            assert!(!contains(r"[\W]", flags, character));
            assert!(contains(r"[^\W]", flags, character));
            assert!(contains(r"[\w]", flags, character));
        }
        assert!(contains(r"[\W]", flags, '!'));
        assert!(!contains(r"[^\W]", flags, '!'));
    }
    assert!(contains(r"[\W]", "i", 'K'));
    assert!(!contains(r"[\W]", "i", 'k'));
    assert!(!contains(r"[[\W]&&[k]]", "vi", 'k'));
    assert!(contains(r"[[^\W]&&[k]]", "vi", 'K'));
}

#[test]
fn unicode_set_operations_fold_each_operand_before_combining_them() {
    for character in ['a', 'A'] {
        assert!(contains("[a&&A]", "vi", character));
        assert!(!contains("[a--A]", "vi", character));
        assert!(!contains("[A--a]", "vi", character));
        assert!(!contains("[^a&&A]", "vi", character));
        assert!(contains("[^a--A]", "vi", character));
        assert!(contains("[[a-z]&&[A-Z]]", "vi", character));
        assert!(!contains("[[a-z]--[A-Z]]", "vi", character));
    }
    assert!(!contains("[a&&A]", "v", 'a'));
    assert!(contains("[a--A]", "v", 'a'));
    assert!(contains("[[k]&&[K]]", "vi", 'K'));
    assert!(!contains("[[k]--[K]]", "vi", 'k'));
    assert!(!contains("[[^k]&&[K]]", "vi", 'k'));
    assert!(contains("[[^k]&&[s]]", "vi", 'ſ'));
}

#[test]
fn property_complements_distinguish_unicode_and_unicode_sets_fold_order() {
    for pattern in [r"\P{Lowercase_Letter}", r"[\P{Lowercase_Letter}]"] {
        for character in ['a', 'A', 'ſ', 'K'] {
            assert!(contains(pattern, "ui", character));
            assert!(!contains(pattern, "vi", character));
        }
        assert!(contains(pattern, "ui", '!'));
        assert!(contains(pattern, "vi", '!'));
    }
    assert!(!contains(
        r"[\P{Lowercase_Letter}&&\p{Uppercase_Letter}]",
        "vi",
        'A'
    ));
    assert!(contains(
        r"[\p{Lowercase_Letter}&&\p{Uppercase_Letter}]",
        "vi",
        'A'
    ));
    assert!(!contains(r"[^\P{Lowercase_Letter}]", "ui", 'a'));
    assert!(contains(r"[^\P{Lowercase_Letter}]", "vi", 'a'));
}

#[test]
fn case_folding_keeps_class_string_gaps_and_later_early_errors_explicit() {
    for pattern in [r"[\q{a}&&A]", r"[\q{ab}--\q{AB}]", r"[a--[\q{A}]]"] {
        let error = RegExpProgram::compile(pattern, "vi").unwrap_err();
        assert_eq!(error.kind, RegExpCompileErrorKind::UnsupportedFeature);
    }
    for pattern in [r"[\q{a}&&]", r"[\q{a}--[z-a]]", r"[^\q{ab}]"] {
        let error = RegExpProgram::compile(pattern, "vi").unwrap_err();
        assert_eq!(error.kind, RegExpCompileErrorKind::InvalidSyntax);
    }
}

#[test]
fn decimal_backreferences_consume_the_whole_decimal_escape() {
    let program = RegExpProgram::compile(r"(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)(k)(l)\12", "u").unwrap();
    let reference = program
        .instructions
        .iter()
        .find(|instruction| instruction.opcode == REGEXP_OPCODE_NUMBERED_BACKREFERENCE)
        .unwrap();
    assert_eq!(reference.operand0, 12);
    let octal = RegExpProgram::compile(r"(a)\10", "").unwrap();
    assert!(!octal
        .instructions
        .iter()
        .any(|instruction| instruction.opcode == REGEXP_OPCODE_NUMBERED_BACKREFERENCE));
    assert!(RegExpProgram::compile(r"(a)\10", "u").is_err());
}

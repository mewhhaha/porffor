use lila_ir::{
    RegExpCompileErrorKind, RegExpInstruction, RegExpProgram, REGEXP_OPCODE_LITERAL_ASCII,
    REGEXP_OPCODE_PROGRESS_CHECK, REGEXP_OPCODE_PROGRESS_SPLIT, REGEXP_OPCODE_WORD_BOUNDARY,
};

fn boundaries(program: &RegExpProgram) -> Vec<&RegExpInstruction> {
    program
        .instructions
        .iter()
        .filter(|instruction| instruction.opcode == REGEXP_OPCODE_WORD_BOUNDARY)
        .collect()
}

fn word_contains(
    program: &RegExpProgram,
    instruction: &RegExpInstruction,
    character: char,
) -> bool {
    let start = instruction.operand0 as usize;
    let end = start + (instruction.operand1 >> 1) as usize;
    program.ranges[start..end]
        .iter()
        .any(|(start, end)| (*start..=*end).contains(&(character as u32)))
}

#[test]
fn assertions_encode_opposite_polarities_over_one_word_set() {
    for flags in ["", "u", "v"] {
        let program = RegExpProgram::compile(r"\b\B", flags).unwrap();
        let assertions = boundaries(&program);
        assert_eq!(assertions.len(), 2);
        assert_eq!(assertions[0].operand0, assertions[1].operand0);
        assert_eq!(assertions[0].operand1 & 1, 0);
        assert_eq!(assertions[1].operand1 & 1, 1);
        for character in ['a', 'Z', '0', '_'] {
            assert!(word_contains(&program, assertions[0], character));
        }
        for character in ['!', 'é', 'ſ', 'K', '𐐀'] {
            assert!(!word_contains(&program, assertions[0], character));
        }
    }
}

#[test]
fn scoped_unicode_ignore_case_controls_the_word_character_closure() {
    for flags in ["u", "v", "ui", "vi"] {
        let program = RegExpProgram::compile(r"(?i:\b)(?-i:\B)", flags).unwrap();
        let assertions = boundaries(&program);
        assert_eq!(assertions.len(), 2);
        for character in ['ſ', 'K'] {
            assert!(word_contains(&program, assertions[0], character));
            assert!(!word_contains(&program, assertions[1], character));
        }
        for character in ['ı', 'İ', 'ß', 'ẞ'] {
            assert!(!word_contains(&program, assertions[0], character));
        }
    }
    let legacy = RegExpProgram::compile(r"(?i:\b)", "").unwrap();
    for character in ['ſ', 'K'] {
        assert!(!word_contains(&legacy, boundaries(&legacy)[0], character));
    }
}

#[test]
fn bare_assertions_reject_quantifiers_in_every_grammar() {
    for pattern in [r"\b?", r"\B*", r"\b+", r"\B{1}", r"\b{0,2}?", r"\B{2,}"] {
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
fn grouped_assertions_use_nullable_repeat_progress_guards() {
    for pattern in [r"(?:\b)*", r"(\B)+", r"(?:(\b)|x)+"] {
        let program = RegExpProgram::compile(pattern, "").unwrap();
        assert!(program
            .instructions
            .iter()
            .any(|instruction| { instruction.opcode == REGEXP_OPCODE_PROGRESS_SPLIT }));
        assert!(program
            .instructions
            .iter()
            .any(|instruction| { instruction.opcode == REGEXP_OPCODE_PROGRESS_CHECK }));
    }
}

#[test]
fn lookbehind_admits_boundary_assertions_in_nested_directions() {
    for pattern in [r"(?<=\bword\b)", r"(?<!\Bword)", r"(?<=a(?=\Bb))b"] {
        for flags in ["", "u", "v"] {
            let program = RegExpProgram::compile(pattern, flags).unwrap();
            assert!(!boundaries(&program).is_empty());
        }
    }
}

#[test]
fn character_class_backspace_remains_a_consuming_atom() {
    for flags in ["", "u", "v"] {
        let program = RegExpProgram::compile(r"[\b]+", flags).unwrap();
        assert!(boundaries(&program).is_empty());
        assert!(
            program.instructions.iter().any(|instruction| {
                (instruction.opcode == REGEXP_OPCODE_LITERAL_ASCII && instruction.operand0 == 8)
                    || instruction.positive_ascii_class_contains(8)
            }) || program.ranges.iter().any(|range| *range == (8, 8))
        );
    }
}

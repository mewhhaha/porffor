use lila_ir::{
    RegExpCompileErrorKind, RegExpProgram, REGEXP_OPCODE_LITERAL_CODE_POINT,
    REGEXP_OPCODE_UNICODE_PROPERTY,
};

fn contains(pattern: &str, flags: &str, character: u32) -> bool {
    let program = RegExpProgram::compile(pattern, flags)
        .unwrap_or_else(|error| panic!("/{pattern}/{flags}: {error}"));
    let instruction = program.instructions[0];
    match instruction.opcode {
        REGEXP_OPCODE_LITERAL_CODE_POINT => instruction.operand0 == u64::from(character),
        REGEXP_OPCODE_UNICODE_PROPERTY => {
            let first = instruction.operand0 as usize;
            let count = (instruction.operand1 >> 1) as usize;
            program.ranges[first..first + count]
                .iter()
                .any(|&(start, end)| (start..=end).contains(&character))
                ^ (instruction.operand1 & 1 != 0)
        }
        opcode => panic!("unexpected class membership opcode {opcode} for /{pattern}/{flags}"),
    }
}

#[test]
fn legacy_raw_astral_members_have_the_same_units_as_escaped_surrogates() {
    for flags in ["", "i"] {
        let raw = RegExpProgram::compile("[😀]", flags).unwrap();
        let escaped = RegExpProgram::compile(r"[\uD83D\uDE00]", flags).unwrap();
        let identity = RegExpProgram::compile(r"[\😀]", flags).unwrap();
        assert_eq!(raw.instructions, escaped.instructions);
        assert_eq!(raw.ranges, escaped.ranges);
        assert_eq!(raw.ranges, identity.ranges);
        assert_eq!(raw.ranges, [(0xd83d, 0xd83d), (0xde00, 0xde00)]);
        assert!(!contains("[😀]", flags, 0x1f600));
        assert!(contains("[^😀]", flags, 0xde01));
        assert!(!contains("[^😀]", flags, 0xd83d));
    }
    for flags in ["u", "ui", "v", "vi"] {
        assert!(contains("[😀]", flags, 0x1f600));
        assert!(!contains("[😀]", flags, 0xd83d));
        assert!(!contains("[😀]", flags, 0xde00));
    }
}

#[test]
fn raw_astral_range_endpoints_follow_utf16_class_grammar() {
    for flags in ["", "i"] {
        for (raw, escaped) in [
            ("[a-😀]", r"[a-\uD83D\uDE00]"),
            (r"[😀-\uFFFF]", r"[\uD83D\uDE00-\uFFFF]"),
            (r"[\w-😀]", r"[\w-\uD83D\uDE00]"),
        ] {
            let raw = RegExpProgram::compile(raw, flags).unwrap();
            let escaped = RegExpProgram::compile(escaped, flags).unwrap();
            assert_eq!(raw.instructions, escaped.instructions);
            assert_eq!(raw.ranges, escaped.ranges);
        }
        assert!(contains("[a-😀]", flags, 0xde00));
        assert!(!contains("[a-😀]", flags, 0xde01));
        assert!(contains(r"[😀-\uFFFF]", flags, 0xd83d));
        assert!(contains(r"[😀-\uFFFF]", flags, 0xde01));
        assert!(!contains(r"[😀-\uFFFF]", flags, u32::from('x')));
        for invalid in ["[😀-😁]", "[a-😀-z]"] {
            assert_eq!(
                RegExpProgram::compile(invalid, flags).unwrap_err().kind,
                RegExpCompileErrorKind::InvalidSyntax
            );
        }
    }
    for flags in ["u", "ui"] {
        assert!(contains("[😀-😁]", flags, 0x1f600));
        assert!(contains("[😀-😁]", flags, 0x1f601));
        assert!(RegExpProgram::compile("[a-😀-z]", flags).is_ok());
        assert_eq!(
            RegExpProgram::compile(r"[😀-\uFFFF]", flags)
                .unwrap_err()
                .kind,
            RegExpCompileErrorKind::InvalidSyntax
        );
    }
}

#[test]
fn pooled_class_complements_and_folding_keep_their_existing_character_sets() {
    for pattern in [r"[\S]", "[^é]", r"[\D]", r"[\W]"] {
        for unit in [0xd83d, 0xde00, 0xd800, 0xdfff] {
            assert!(contains(pattern, "", unit));
        }
    }
    assert!(contains("[é]", "i", u32::from('É')));
    assert!(!contains("[^é]", "i", u32::from('É')));
    assert!(!contains(r"[\u006b]", "i", u32::from('K')));
    assert!(contains("[é😀]", "i", 0xd83d));
    assert!(contains("[é😀]", "i", 0xde00));
    for flags in ["ui", "vi"] {
        assert!(contains("[k]", flags, u32::from('K')));
        assert!(contains("[𐐀]", flags, 0x10428));
        assert!(!contains("[^𐐀]", flags, 0x10428));
    }
}

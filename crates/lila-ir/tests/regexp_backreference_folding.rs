use lila_ir::{
    RegExpCaseFolding, RegExpCompileErrorKind, RegExpInstruction, RegExpProgram,
    REGEXP_BACKREFERENCE_IGNORE_CASE, REGEXP_BACKREFERENCE_NONEMPTY,
    REGEXP_OPCODE_NAMED_BACKREFERENCE, REGEXP_OPCODE_NUMBERED_BACKREFERENCE,
};

fn references(pattern: &str, flags: &str) -> Vec<RegExpInstruction> {
    RegExpProgram::compile(pattern, flags)
        .unwrap_or_else(|error| panic!("/{pattern}/{flags}: {error}"))
        .instructions
        .into_iter()
        .filter(|instruction| {
            matches!(
                instruction.opcode,
                REGEXP_OPCODE_NAMED_BACKREFERENCE | REGEXP_OPCODE_NUMBERED_BACKREFERENCE
            )
        })
        .collect()
}

#[test]
fn each_reference_encodes_its_own_resolved_ignore_case_modifier() {
    for flags in ["", "u", "v"] {
        let instructions = references(r"(a)(?i:\1)(?-i:\1)(?i:(?-i:\1)\1)\1", flags);
        assert_eq!(
            instructions
                .iter()
                .map(|instruction| instruction.operand1)
                .collect::<Vec<_>>(),
            [2, 0, 0, 2, 0]
        );
        let named = references(
            r"(?<a>a)(?i:\k<a>)(?-i:\k<a>)(?i:(?-i:\k<a>)\k<a>)\k<a>",
            flags,
        );
        assert_eq!(
            named
                .iter()
                .map(|instruction| instruction.operand1)
                .collect::<Vec<_>>(),
            [2, 0, 0, 2, 0]
        );
    }
    assert_eq!(references(r"(?i:(a))\1", "")[0].operand1, 0);
    assert_eq!(references(r"(?-i:(a))\1", "i")[0].operand1, 2);
    assert_eq!(references(r"(?i:(?<a>a))\k<a>", "")[0].operand1, 0);
    assert_eq!(references(r"(?-i:(?<a>a))\k<a>", "i")[0].operand1, 2);
}

#[test]
fn reverse_references_retain_scoped_folding_and_numbered_nullability() {
    for flags in ["i", "ui", "vi"] {
        assert_eq!(references(r"(?<=\1(a))x", flags)[0].operand1, 2);
        assert_eq!(references(r"(?<=(?-i:\1)(a))x", flags)[0].operand1, 0);
        assert_eq!(references(r"(?<=\k<a>(?<a>a))x", flags)[0].operand1, 2);
        assert_eq!(
            references(r"(?<=(?-i:\k<a>)(?<a>a))x", flags)[0].operand1,
            0
        );
        assert_eq!(references(r"(a*)\1*", flags)[0].operand1, 2);
        assert_eq!(references(r"(a)\1*", flags)[0].operand1, 2);
        assert_eq!(references(r"()\1", flags)[0].operand1, 2);
    }
}

#[test]
fn constructors_keep_named_reserved_bits_distinct_from_numbered_proofs() {
    assert_eq!(REGEXP_BACKREFERENCE_NONEMPTY, 1);
    assert_eq!(REGEXP_BACKREFERENCE_IGNORE_CASE, 2);
    for folding in [
        RegExpCaseFolding::Sensitive,
        RegExpCaseFolding::Legacy,
        RegExpCaseFolding::Unicode,
    ] {
        let expected = if folding == RegExpCaseFolding::Sensitive {
            0
        } else {
            2
        };
        assert_eq!(
            RegExpInstruction::named_backreference(7, folding).operand1,
            expected
        );
        assert_eq!(
            RegExpInstruction::numbered_backreference(9, folding).operand1,
            expected
        );
        assert_eq!(
            RegExpInstruction::nonempty_numbered_backreference(9, folding).operand1,
            expected | 1
        );
    }
}

fn canonicalize(folding: RegExpCaseFolding, character: u32) -> u32 {
    let mappings = folding.mappings();
    mappings
        .binary_search_by_key(&character, |&(source, _)| source)
        .map_or(character, |index| mappings[index].1)
}

#[test]
fn shared_tables_are_sorted_nonidentity_idempotent_canonicalization_mappings() {
    assert!(RegExpCaseFolding::Sensitive.mappings().is_empty());
    assert_eq!(RegExpCaseFolding::Unicode.mappings().len(), 1512);
    for folding in [RegExpCaseFolding::Legacy, RegExpCaseFolding::Unicode] {
        let mappings = folding.mappings();
        assert!(!mappings.is_empty());
        assert!(mappings.windows(2).all(|rows| rows[0].0 < rows[1].0));
        for &(source, canonical) in mappings {
            assert_ne!(source, canonical);
            assert_eq!(canonicalize(folding, canonical), canonical);
            if folding == RegExpCaseFolding::Legacy {
                assert!(source <= 0xffff && canonical <= 0xffff);
                assert!(source < 128 || canonical >= 128);
            }
        }
    }
    for (source, expected) in [
        (0x41, 0x61),
        (0x212a, 0x6b),
        (0x17f, 0x73),
        (0x1e9e, 0xdf),
        (0x10400, 0x10428),
    ] {
        assert_eq!(canonicalize(RegExpCaseFolding::Unicode, source), expected);
    }
    for (source, expected) in [
        (0x61, 0x41),
        (0xe9, 0xc9),
        (0x3c2, 0x3a3),
        (0xdf, 0xdf),
        (0x212a, 0x212a),
        (0x17f, 0x17f),
        (0x10428, 0x10428),
    ] {
        assert_eq!(canonicalize(RegExpCaseFolding::Legacy, source), expected);
    }
    for folding in [RegExpCaseFolding::Legacy, RegExpCaseFolding::Unicode] {
        for character in [0x130, 0x131, 0xd800, 0xdfff] {
            assert_eq!(canonicalize(folding, character), character);
        }
    }
}

#[test]
fn scoped_folding_does_not_bypass_named_or_decimal_reference_early_errors() {
    for (pattern, flags) in [
        (r"(?i:\k<missing>)", "u"),
        (r"(?<a>a)(?i:\k<missing>)", ""),
        (r"(?i:\2)(a)", "u"),
        (r"(?m:\2)(a)", "u"),
        (r"(?s:\2)(a)", "v"),
        (r"(?-ims:\2)(a)", "u"),
        (r"(?i:(?m:(a)))\2", "v"),
        (r"(?i:\k<a)(?<a>a)", "v"),
    ] {
        assert_eq!(
            RegExpProgram::compile(pattern, flags).unwrap_err().kind,
            RegExpCompileErrorKind::InvalidSyntax,
            "/{pattern}/{flags}"
        );
    }
    assert!(references(r"(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)(?i:\10)", "u")
        .iter()
        .any(|instruction| instruction.operand0 == 10 && instruction.operand1 == 2));
    assert!(references(r"(?i:\2)(a)", "").is_empty());
    assert_eq!(references(r"(?i:\1)(a)", "u")[0].operand0, 1);
}

#[test]
fn references_do_not_infer_capture_participation_from_nonempty_bodies() {
    for pattern in [
        r"(a)?\1*",
        r"(?:(a)|b)\1*",
        r"(?!(a))\1*",
        r"(a)\1*",
        r"\1*(a)",
    ] {
        for flags in ["", "i", "u", "ui", "v", "vi"] {
            let program = RegExpProgram::compile(pattern, flags).unwrap();
            assert!(program.instructions.iter().filter(|instruction| instruction.opcode == REGEXP_OPCODE_NUMBERED_BACKREFERENCE).all(|instruction| instruction.operand1 & REGEXP_BACKREFERENCE_NONEMPTY == 0), "/{pattern}/{flags}");
            assert!(
                program
                    .instructions
                    .iter()
                    .any(|instruction| instruction.opcode == lila_ir::REGEXP_OPCODE_PROGRESS_SPLIT),
                "/{pattern}/{flags}"
            );
        }
    }
}

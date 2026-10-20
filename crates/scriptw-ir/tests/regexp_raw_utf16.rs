use scriptw_ir::{
    decode_js_string_utf16, encode_js_string_utf16, RegExpCompileErrorKind, RegExpInstruction,
    RegExpProgram, JS_STRING_SURROGATE_SENTINEL, REGEXP_OPCODE_LITERAL_CODE_POINT,
    REGEXP_OPCODE_UNICODE_PROPERTY,
};

fn units(source: &str) -> Vec<u16> {
    source.encode_utf16().collect()
}
fn joined(parts: &[&[u16]]) -> Vec<u16> {
    parts.concat()
}
fn contains(program: &RegExpProgram, point: u32) -> bool {
    let atom = program.instructions[0];
    match atom.opcode {
        REGEXP_OPCODE_LITERAL_CODE_POINT => atom.operand0 == u64::from(point),
        REGEXP_OPCODE_UNICODE_PROPERTY => {
            let first = atom.operand0 as usize;
            let count = (atom.operand1 >> 1) as usize;
            program.ranges[first..first + count]
                .iter()
                .any(|&(start, end)| (start..=end).contains(&point))
                ^ (atom.operand1 & 1 != 0)
        }
        opcode => panic!("unexpected class opcode {opcode}"),
    }
}

#[test]
fn semantic_pattern_units_preserve_raw_fixed_and_braced_atom_ownership() {
    let fixed_high = units(r"\uD83D");
    let fixed_low = units(r"\uDC38");
    let braced_high = units(r"\u{D83D}");
    let braced_low = units(r"\u{DC38}");
    let high = RegExpInstruction::literal_code_point(0xd83d);
    let low = RegExpInstruction::literal_code_point(0xdc38);
    let frog = RegExpInstruction::literal_code_point(0x1f438);
    let accept = RegExpInstruction::accept();
    for flags in ["", "u", "v"] {
        let raw = RegExpProgram::compile_utf16(&[0xd83d, 0xdc38], flags).unwrap();
        assert_eq!(
            raw.instructions,
            if flags.is_empty() {
                vec![high, low, accept]
            } else {
                vec![frog, accept]
            }
        );
        for pattern in [
            joined(&[&fixed_high, &[0xdc38]]),
            joined(&[&[0xd83d], &fixed_low]),
        ] {
            let compiled = RegExpProgram::compile_utf16(&pattern, flags).unwrap();
            assert_eq!(
                compiled.instructions,
                vec![high, low, accept],
                "mixed pattern {pattern:?}/{flags}"
            );
        }
        let escaped =
            RegExpProgram::compile_utf16(&joined(&[&fixed_high, &fixed_low]), flags).unwrap();
        assert_eq!(
            escaped.instructions,
            if flags.is_empty() {
                vec![high, low, accept]
            } else {
                vec![frog, accept]
            }
        );
        if !flags.is_empty() {
            for pattern in [
                joined(&[&braced_high, &[0xdc38]]),
                joined(&[&[0xd83d], &braced_low]),
            ] {
                assert_eq!(
                    RegExpProgram::compile_utf16(&pattern, flags)
                        .unwrap()
                        .instructions,
                    vec![high, low, accept]
                );
            }
        }
    }
}

#[test]
fn semantic_raw_patterns_quantify_the_actual_last_grammar_atom() {
    let high = RegExpInstruction::literal_code_point(0xd83d);
    let low = RegExpInstruction::literal_code_point(0xdc38);
    let accept = RegExpInstruction::accept();
    for flags in ["", "u", "v"] {
        let raw = RegExpProgram::compile_utf16(&[0xd83d, 0xdc38, b'?' as u16], flags).unwrap();
        assert_eq!(
            raw.instructions,
            if flags.is_empty() {
                vec![high, RegExpInstruction::split(2, 3), low, accept]
            } else {
                vec![
                    RegExpInstruction::split(1, 2),
                    RegExpInstruction::literal_code_point(0x1f438),
                    accept,
                ]
            }
        );
        for pattern in [
            joined(&[&units(r"\uD83D"), &[0xdc38, b'?' as u16]]),
            joined(&[&[0xd83d], &units(r"\uDC38?")]),
        ] {
            assert_eq!(
                RegExpProgram::compile_utf16(&pattern, flags)
                    .unwrap()
                    .instructions,
                vec![high, RegExpInstruction::split(2, 3), low, accept]
            );
        }
    }
}

#[test]
fn semantic_class_units_do_not_merge_mixed_atoms_or_change_range_order() {
    for flags in ["", "u", "v"] {
        let raw = RegExpProgram::compile_utf16(&[91, 0xd83d, 0xdc38, 93], flags).unwrap();
        assert_eq!(contains(&raw, 0x1f438), !flags.is_empty());
        assert_eq!(contains(&raw, 0xd83d), flags.is_empty());
        assert_eq!(contains(&raw, 0xdc38), flags.is_empty());
        for middle in [
            joined(&[&units(r"\uD83D"), &[0xdc38]]),
            joined(&[&[0xd83d], &units(r"\uDC38")]),
            joined(&[&units(r"\uDC38"), &[0xd83d]]),
        ] {
            let pattern = joined(&[&[91], &middle, &[93]]);
            let compiled = RegExpProgram::compile_utf16(&pattern, flags).unwrap();
            assert!(!contains(&compiled, 0x1f438));
            assert!(contains(&compiled, 0xd83d));
            assert!(contains(&compiled, 0xdc38));
        }
        let ordered = RegExpProgram::compile_utf16(
            &joined(&[&[91, 0xd83d, 45], &units(r"\uDC38"), &[93]]),
            flags,
        )
        .unwrap();
        assert!(contains(&ordered, 0xda00));
        assert_eq!(
            RegExpProgram::compile_utf16(
                &joined(&[&[91, 0xdc38, 45], &units(r"\uD83D"), &[93]]),
                flags
            )
            .unwrap_err()
            .kind,
            RegExpCompileErrorKind::InvalidSyntax
        );
    }
}

#[test]
fn semantic_source_codec_retains_every_unit_and_the_marker_as_user_data() {
    let every_unit = (0..=u16::MAX).collect::<Vec<_>>();
    assert_eq!(
        decode_js_string_utf16(&encode_js_string_utf16(&every_unit)),
        every_unit
    );
    let text = format!("{JS_STRING_SURROGATE_SENTINEL}D83D");
    let raw = units(&text);
    assert_eq!(decode_js_string_utf16(&encode_js_string_utf16(&raw)), raw);
    assert_eq!(
        RegExpProgram::compile_utf16(&raw, "u").unwrap(),
        RegExpProgram::compile(&text, "u").unwrap()
    );
    // A scalar before a later raw surrogate is still a valid source character.
    let pattern = joined(&[&units("é"), &[0xd83d]]);
    assert_eq!(
        RegExpProgram::compile_utf16(&pattern, "u")
            .unwrap()
            .instructions,
        vec![
            RegExpInstruction::literal_code_point(0xe9),
            RegExpInstruction::literal_code_point(0xd83d),
            RegExpInstruction::accept()
        ]
    );
    for pattern in [
        joined(&[&units(r"\p{"), &[0xd83d], &units("}")]),
        joined(&[&units("(?<"), &[0xd83d], &units(">a)")]),
    ] {
        assert_eq!(
            RegExpProgram::compile_utf16(&pattern, "u")
                .unwrap_err()
                .kind,
            RegExpCompileErrorKind::InvalidSyntax
        );
    }
}

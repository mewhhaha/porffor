use lila_ir::{
    regexp_unicode_property_catalog, RegExpCompileErrorKind, RegExpProgram,
    RegExpUnicodePropertyCatalogValue, REGEXP_OPCODE_ACCEPT, REGEXP_OPCODE_LITERAL_ASCII,
    REGEXP_OPCODE_UNICODE_PROPERTY,
};

fn property_member(program: &RegExpProgram, code_point: u32) -> bool {
    let atom = &program.instructions[0];
    assert_eq!(atom.opcode, REGEXP_OPCODE_UNICODE_PROPERTY);
    let first = atom.operand0 as usize;
    let last = first + (atom.operand1 >> 1) as usize;
    program.ranges[first..last]
        .iter()
        .any(|(start, end)| (*start..=*end).contains(&code_point))
        ^ (atom.operand1 & 1 != 0)
}

#[test]
fn icu_only_mixed_case_binary_names_are_syntax_errors_in_both_unicode_modes() {
    for name in [
        "Id_Start",
        "Id_Continue",
        "Ids_Binary_Operator",
        "Ids_Trinary_Operator",
    ] {
        for flags in ["u", "v", "ui", "vi"] {
            for (pattern, offset) in [
                (format!(r"\p{{{name}}}"), 0),
                (format!(r"\P{{{name}}}"), 0),
                (format!(r"[\p{{{name}}}]"), 1),
                (format!(r"[\P{{{name}}}]"), 1),
            ] {
                let error = RegExpProgram::compile(&pattern, flags)
                    .expect_err("ICU's mixed-case spelling is not an ECMAScript alias");
                assert_eq!(
                    error.kind,
                    RegExpCompileErrorKind::InvalidSyntax,
                    "{pattern}/{flags}"
                );
                assert_eq!(error.offset, offset, "{pattern}/{flags}");
            }
        }
    }
}

#[test]
fn normative_binary_names_and_aliases_share_actual_range_and_complement_semantics() {
    for (name, alias, witness) in [
        ("ID_Start", "IDS", u32::from(b'a')),
        ("ID_Continue", "IDC", u32::from(b'0')),
        ("IDS_Binary_Operator", "IDSB", 0x2ff0),
        ("IDS_Trinary_Operator", "IDST", 0x2ff2),
    ] {
        for flags in ["u", "v"] {
            let canonical = RegExpProgram::compile(&format!(r"\p{{{name}}}"), flags).unwrap();
            let alias_program = RegExpProgram::compile(&format!(r"\p{{{alias}}}"), flags).unwrap();
            assert_eq!(
                canonical.instructions, alias_program.instructions,
                "{name}/{alias}/{flags}"
            );
            assert_eq!(
                canonical.ranges, alias_program.ranges,
                "{name}/{alias}/{flags}"
            );
            assert!(property_member(&canonical, witness), "{name}/{flags}");
            assert!(
                !property_member(&canonical, u32::from(b'!')),
                "{name}/{flags}"
            );
            for spelling in [name, alias] {
                for pattern in [format!(r"\P{{{spelling}}}"), format!(r"[\P{{{spelling}}}]")] {
                    let complement = RegExpProgram::compile(&pattern, flags).unwrap();
                    assert!(!property_member(&complement, witness), "{pattern}/{flags}");
                    assert!(
                        property_member(&complement, u32::from(b'!')),
                        "{pattern}/{flags}"
                    );
                }
            }
        }
    }
    for spelling in ["ID_Start", "IDS", "ID_Continue", "IDC"] {
        let program = RegExpProgram::compile(&format!(r"\p{{{spelling}}}"), "u").unwrap();
        assert!(
            property_member(&program, 0x16ea0),
            "Unicode 17 Beria Erfe: {spelling}"
        );
    }
}

#[test]
fn exact_general_category_script_and_binary_families_keep_their_value_domains() {
    for flags in ["u", "v"] {
        for (spellings, witness, nonmember) in [
            (
                ["L", "Letter", "General_Category=L", "gc=Letter"].as_slice(),
                u32::from(b'a'),
                u32::from(b'0'),
            ),
            (
                ["Script=Han", "Script=Hani", "sc=Han", "sc=Hani"].as_slice(),
                0x4e00,
                u32::from(b'a'),
            ),
            (
                [
                    "Script_Extensions=Hira",
                    "Script_Extensions=Hiragana",
                    "scx=Hira",
                    "scx=Hiragana",
                ]
                .as_slice(),
                0x30fc,
                u32::from(b'a'),
            ),
            (["White_Space", "space"].as_slice(), 0x20, u32::from(b'a')),
            (
                ["cntrl", "Control", "gc=Cc", "General_Category=cntrl"].as_slice(),
                0,
                u32::from(b'a'),
            ),
            (
                ["digit", "Nd", "gc=Decimal_Number"].as_slice(),
                u32::from(b'0'),
                u32::from(b'a'),
            ),
            (
                ["Combining_Mark", "M", "gc=Mark"].as_slice(),
                0x300,
                u32::from(b'a'),
            ),
        ] {
            for spelling in spellings {
                let program = RegExpProgram::compile(&format!(r"\p{{{spelling}}}"), flags).unwrap();
                assert!(property_member(&program, witness), "{spelling}/{flags}");
                assert!(!property_member(&program, nonmember), "{spelling}/{flags}");
            }
        }
        let any = RegExpProgram::compile(r"\p{Any}", flags).unwrap();
        assert!(
            property_member(&any, 0xd800),
            "Any includes lone surrogates: {flags}"
        );
        assert!(
            property_member(&any, 0x10ffff),
            "Any includes the maximum code point: {flags}"
        );
        for spelling in [
            "general_category=L",
            "GC=L",
            "script=Han",
            "Sc=Han",
            "scx=han",
            "General_Category=Emoji",
            "General_Category=Alpha",
            "Script=Letter",
            "Script_Extensions=Letter",
            "Alphabetic=Yes",
            "Any=Yes",
            "RGI_Emoji=Yes",
            "Han",
            "WSpace",
            "",
            "gc=",
            "General_Category=",
            "Script=",
            "scx=",
            "L&",
            "gc=L&",
            "gc= Letter",
            "Script=Old-Persian",
            "Script=Latin=Greek",
            "gc=Lé",
            "sc=Hàn",
        ] {
            let error = RegExpProgram::compile(&format!(r"\p{{{spelling}}}"), flags).unwrap_err();
            assert_eq!(
                error.kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{spelling}/{flags}"
            );
        }
    }
}

#[test]
fn all_committed_unicode17_script_names_keep_both_script_families_and_exact_aliases() {
    for (name, alias, witness) in [
        ("Beria_Erfe", "Berf", 0x16ea0),
        ("Sidetic", "Sidt", 0x10940),
        ("Tai_Yo", "Tayo", 0x1e6c0),
        ("Tolong_Siki", "Tols", 0x11db0),
    ] {
        for flags in ["u", "v"] {
            for family in ["Script", "sc", "Script_Extensions", "scx"] {
                let canonical =
                    RegExpProgram::compile(&format!(r"\p{{{family}={name}}}"), flags).unwrap();
                let short =
                    RegExpProgram::compile(&format!(r"\p{{{family}={alias}}}"), flags).unwrap();
                assert_eq!(
                    canonical.ranges, short.ranges,
                    "{family}={name}/{alias}/{flags}"
                );
                assert!(property_member(&canonical, witness));
                assert!(!property_member(&canonical, u32::from(b'a')));
                let bad_case = format!(r"\p{{{family}={}}}", name.to_lowercase());
                assert_eq!(
                    RegExpProgram::compile(&bad_case, flags).unwrap_err().kind,
                    RegExpCompileErrorKind::InvalidSyntax
                );
            }
            for family in ["gc", "General_Category"] {
                assert_eq!(
                    RegExpProgram::compile(&format!(r"\p{{{family}={name}}}"), flags)
                        .unwrap_err()
                        .kind,
                    RegExpCompileErrorKind::InvalidSyntax
                );
            }
            assert_eq!(
                RegExpProgram::compile(&format!(r"\p{{{name}}}"), flags)
                    .unwrap_err()
                    .kind,
                RegExpCompileErrorKind::InvalidSyntax
            );
        }
    }
}

#[test]
fn string_properties_and_legacy_identity_escapes_keep_their_separate_grammars() {
    for name in ["Basic_Emoji", "Emoji_Keycap_Sequence"] {
        assert!(RegExpProgram::compile(&format!(r"\p{{{name}}}"), "v").is_ok());
        for (pattern, flags) in [
            (format!(r"\p{{{name}}}"), "u"),
            (format!(r"\P{{{name}}}"), "u"),
            (format!(r"\P{{{name}}}"), "v"),
            (format!(r"[^\p{{{name}}}]"), "v"),
        ] {
            assert_eq!(
                RegExpProgram::compile(&pattern, flags).unwrap_err().kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}/{flags}"
            );
        }
    }
    for pattern in [
        r"\p{Id_Start}",
        r"\p{Id_Continue}",
        r"\p{Ids_Binary_Operator}",
        r"\p{Ids_Trinary_Operator}",
        r"\p{}",
        r"\p{Id_Start",
        r"\p{",
    ] {
        let program = RegExpProgram::compile(pattern, "").unwrap();
        let mut literals = Vec::new();
        for instruction in &program.instructions {
            match instruction.opcode {
                REGEXP_OPCODE_LITERAL_ASCII => literals.push(instruction.operand0 as u8),
                REGEXP_OPCODE_ACCEPT => {}
                opcode => panic!("Legacy identity source {pattern} emitted {opcode}"),
            }
        }
        assert_eq!(
            literals,
            pattern.as_bytes()[1..],
            "Legacy identity source {pattern}"
        );
    }
}

#[test]
fn emitted_property_catalog_retains_pinned_codepoint_membership_and_exact_alias_domains() {
    let catalog = regexp_unicode_property_catalog();
    for (name, member, nonmember) in [
        ("Any", 0x10ffff, None),
        ("Assigned", 0x16ea0, Some(0x10ffff)),
        ("gc=Cs", 0xd800, Some(u32::from(b'a'))),
        ("digit", u32::from(b'0'), Some(u32::from(b'a'))),
        (
            "General_Category=Combining_Mark",
            0x300,
            Some(u32::from(b'a')),
        ),
        ("IDS", u32::from(b'a'), Some(u32::from(b'!'))),
        ("sc=Hira", 0x3042, Some(0x30fc)),
        ("Script_Extensions=Hiragana", 0x30fc, Some(u32::from(b'a'))),
        ("Script=Beria_Erfe", 0x16ea0, Some(0x16eb9)),
        ("scx=Berf", 0x16ed3, Some(0x16ed4)),
        ("Script=Sidetic", 0x10940, Some(0x1095a)),
        ("scx=Sidt", 0x10959, Some(0x1095a)),
        ("Script=Tai_Yo", 0x1e6c0, Some(0x1e6df)),
        ("scx=Tayo", 0x1e6ff, Some(0x1e700)),
        ("Script=Tolong_Siki", 0x11db0, Some(0x11ddc)),
        ("scx=Tols", 0x11de9, Some(0x11dea)),
    ] {
        let row = catalog.iter().find(|row| row.name() == name).expect(name);
        let ranges = match row.value() {
            RegExpUnicodePropertyCatalogValue::CodePoints(ranges) => ranges,
            RegExpUnicodePropertyCatalogValue::Strings(_) => {
                panic!("{name} is a code-point property")
            }
        };
        assert!(
            ranges
                .iter()
                .any(|&(first, last)| first <= member && member <= last),
            "{name}: U+{member:X}"
        );
        if let Some(nonmember) = nonmember {
            assert!(
                !ranges
                    .iter()
                    .any(|&(first, last)| first <= nonmember && nonmember <= last),
                "{name}: U+{nonmember:X}"
            );
        }
    }
    for name in [
        "Id_Start",
        "Id_Continue",
        "Ids_Binary_Operator",
        "Ids_Trinary_Operator",
        "Han",
        "Script=Letter",
        "General_Category=Alpha",
        "gc=L&",
        "gc= Letter",
        "sc=Hàn",
    ] {
        assert!(
            catalog.iter().all(|row| row.name() != name),
            "unvalidated key {name}"
        );
    }
    // These are actual property members, not UTF-16 spellings or row tags.
    // Pinned set sizes detect incomplete provider payloads; strict prefixes
    // distinguish indivisible string members from code-point membership.
    for (name, count, member, nonmember) in [
        ("Basic_Emoji", 1400, vec![0x231a], vec![]),
        (
            "Emoji_Keycap_Sequence",
            12,
            vec![0x30, 0xfe0f, 0x20e3],
            vec![0x30, 0xfe0f],
        ),
        (
            "RGI_Emoji_Flag_Sequence",
            259,
            vec![0x1f1e6, 0x1f1e8],
            vec![0x1f1e6],
        ),
        (
            "RGI_Emoji_Modifier_Sequence",
            665,
            vec![0x261d, 0x1f3fb],
            vec![0x261d],
        ),
        (
            "RGI_Emoji_Tag_Sequence",
            3,
            vec![
                0x1f3f4, 0xe0067, 0xe0062, 0xe0065, 0xe006e, 0xe0067, 0xe007f,
            ],
            vec![0x1f3f4, 0xe0067, 0xe0062],
        ),
        (
            "RGI_Emoji_ZWJ_Sequence",
            1614,
            vec![0x1f468, 0x200d, 0x1f469, 0x200d, 0x1f466],
            vec![0x1f468, 0x200d, 0x1f469],
        ),
        ("RGI_Emoji", 3953, vec![0x30, 0xfe0f, 0x20e3], vec![0x30]),
    ] {
        let row = catalog.iter().find(|row| row.name() == name).expect(name);
        let RegExpUnicodePropertyCatalogValue::Strings(sequences) = row.value() else {
            panic!("{name} has string members");
        };
        assert_eq!(sequences.len(), count, "complete pinned set {name}");
        assert!(sequences.binary_search(&member).is_ok(), "member {name}");
        assert!(
            sequences.binary_search(&nonmember).is_err(),
            "strict prefix {name}"
        );
        assert!(
            sequences
                .binary_search(&vec![0xd83c, 0xdde6, 0xd83c, 0xdde8])
                .is_err(),
            "code-point keys must not be split into UTF-16 units: {name}"
        );
    }
}

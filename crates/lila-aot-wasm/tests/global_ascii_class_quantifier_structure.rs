use std::fs;
use std::path::Path;

const STRING_PARENT: &str = include_str!("../src/builtins/string.rs");
const GLOBAL_ASCII_CLASS_QUANTIFIER: &str =
    include_str!("../src/builtins/string/global_ascii_class_quantifier.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn global_ascii_class_quantifier_is_a_private_non_copyable_three_variant_domain() {
    let domain = bounded(
        GLOBAL_ASCII_CLASS_QUANTIFIER,
        "enum GlobalAsciiClassQuantifier {",
        "\n}\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    let variants = domain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();

    assert_eq!(variants, ["DigitOnce,", "DigitTwice,", "NonDigitTwice,"]);
    let declaration_start = GLOBAL_ASCII_CLASS_QUANTIFIER
        .find("enum GlobalAsciiClassQuantifier {")
        .expect("missing quantifier domain");
    let declaration_prefix = &GLOBAL_ASCII_CLASS_QUANTIFIER[..declaration_start];
    let preceding_declaration = declaration_prefix
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .expect("missing preceding declaration");
    assert_eq!(preceding_declaration.trim(), "use super::*;");
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!domain.contains(capability));
        assert!(!GLOBAL_ASCII_CLASS_QUANTIFIER
            .contains(&format!("impl {capability} for GlobalAsciiClassQuantifier")));
    }
    assert!(!GLOBAL_ASCII_CLASS_QUANTIFIER.contains("pub enum GlobalAsciiClassQuantifier"));
    assert!(!GLOBAL_ASCII_CLASS_QUANTIFIER.contains("pub(crate) enum GlobalAsciiClassQuantifier"));
    assert!(!GLOBAL_ASCII_CLASS_QUANTIFIER.contains("pub(super) enum GlobalAsciiClassQuantifier"));
    assert_eq!(
        STRING_PARENT
            .matches("mod global_ascii_class_quantifier;")
            .count(),
        1
    );
    assert!(!STRING_PARENT.contains("GlobalAsciiClassQuantifier"));
    assert!(!STRING_PARENT.contains("global_ascii_class_quantifier::"));
}

#[test]
fn quantifier_emitter_projects_width_and_ascii_polarity_exhaustively() {
    let emitter = bounded(
        GLOBAL_ASCII_CLASS_QUANTIFIER,
        "    fn emit_string_match_global_ascii_class_quantifier_from_string_locals(",
        "\n    }\n}",
    );
    let normalized = without_whitespace(emitter);

    assert!(emitter.contains("class_quantifier: GlobalAsciiClassQuantifier,"));
    assert_eq!(emitter.matches("match &class_quantifier {").count(), 2);
    assert!(normalized.contains(
        "Instruction::I64Const(match&class_quantifier{GlobalAsciiClassQuantifier::DigitOnce=>1,GlobalAsciiClassQuantifier::DigitTwice|GlobalAsciiClassQuantifier::NonDigitTwice=>2,})"
    ));
    assert!(normalized.contains(
        "match&class_quantifier{GlobalAsciiClassQuantifier::DigitOnce|GlobalAsciiClassQuantifier::DigitTwice=>{}GlobalAsciiClassQuantifier::NonDigitTwice=>{function.instruction(&Instruction::I32Eqz);}}"
    ));
    for forbidden in [
        "match_digits",
        "quantifier: i64",
        ": bool",
        "matches!(class_quantifier",
        "if class_quantifier",
        "_ =>",
        "unreachable!",
    ] {
        assert!(!emitter.contains(forbidden));
    }

    assert_eq!(
        emitter
            .matches("self.emit_decode_utf8_scalar_at_index(")
            .count(),
        2
    );
    assert!(normalized.contains(
        "self.emit_decode_utf8_scalar_at_index(src_offset_local,probe_index_local,src_len_local,first_byte_local,codepoint_local,advance_local,temp_local,function,)"
    ));
    assert!(normalized.contains(
        "self.emit_decode_utf8_scalar_at_index(src_offset_local,scan_index_local,src_len_local,first_byte_local,codepoint_local,advance_local,temp_local,function,)"
    ));
    assert!(normalized.contains(
        "Instruction::LocalGet(probe_index_local));function.instruction(&Instruction::LocalGet(advance_local));function.instruction(&Instruction::I64Add);function.instruction(&Instruction::LocalSet(probe_index_local))"
    ));
    assert!(normalized.contains(
        "Instruction::LocalGet(probe_index_local));function.instruction(&Instruction::LocalSet(scan_index_local));function.instruction(&Instruction::Else);self.emit_load_string_byte(src_offset_local,scan_index_local"
    ));
    assert!(normalized.contains(
        "Instruction::LocalGet(scan_index_local));function.instruction(&Instruction::LocalGet(advance_local));function.instruction(&Instruction::I64Add);function.instruction(&Instruction::LocalSet(scan_index_local))"
    ));
    assert!(normalized.contains("ValueKind::Array.tag()asi64"));
    assert!(normalized.contains("ValueKind::Null.tag()asi64"));
}

#[test]
fn exactly_three_patterns_choose_their_named_quantifiers() {
    let producers = bounded(
        STRING_PARENT,
        "self.strings.payload(\"\\\\d{1}\")",
        "self.emit_string_match_global_dot_sequence_from_string_locals(",
    );
    let digit_once = bounded(
        STRING_PARENT,
        "self.strings.payload(\"\\\\d{1}\")",
        "self.strings.payload(\"\\\\d{2}\")",
    );
    let digit_twice = bounded(
        STRING_PARENT,
        "self.strings.payload(\"\\\\d{2}\")",
        "self.strings.payload(\"\\\\D{2}\")",
    );
    let non_digit_twice = bounded(
        STRING_PARENT,
        "self.strings.payload(\"\\\\D{2}\")",
        "self.strings.payload(\".(.).\")",
    );

    assert_eq!(
        digit_once
            .matches("self.emit_string_match_global_ascii_digit_once_from_string_locals(")
            .count(),
        1
    );
    assert_eq!(
        digit_twice
            .matches("self.emit_string_match_global_ascii_digit_twice_from_string_locals(")
            .count(),
        1
    );
    assert_eq!(
        non_digit_twice
            .matches("self.emit_string_match_global_ascii_non_digit_twice_from_string_locals(")
            .count(),
        1
    );
    assert_eq!(
        STRING_PARENT
            .matches("emit_string_match_global_ascii_class_quantifier_from_string_locals(")
            .count(),
        0
    );
    assert!(!producers.contains("true,"));
    assert!(!producers.contains("false,"));

    let child_normalized = without_whitespace(GLOBAL_ASCII_CLASS_QUANTIFIER);
    for mapping in [
        "self.emit_string_match_global_ascii_class_quantifier_from_string_locals(string_local,GlobalAsciiClassQuantifier::DigitOnce,payload_local,tag_local,function,)",
        "self.emit_string_match_global_ascii_class_quantifier_from_string_locals(string_local,GlobalAsciiClassQuantifier::DigitTwice,payload_local,tag_local,function,)",
        "self.emit_string_match_global_ascii_class_quantifier_from_string_locals(string_local,GlobalAsciiClassQuantifier::NonDigitTwice,payload_local,tag_local,function,)",
    ] {
        assert_eq!(child_normalized.matches(mapping).count(), 1);
    }
    assert_eq!(
        GLOBAL_ASCII_CLASS_QUANTIFIER
            .matches("GlobalAsciiClassQuantifier")
            .count(),
        11
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_string_match_global_ascii_class_quantifier_from_string_locals(",
        ),
        4,
        "the private emitter definition and its three semantic wrapper calls must stay inventoried"
    );
}

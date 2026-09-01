const REGEXP_SOURCE: &str = include_str!("../src/regexp.rs");
const BEHAVIOR_SOURCE: &str = include_str!("regexp_lookbehind_polarity.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/regexp-lookbehind-polarity.md");
const TASK: &str = include_str!("../../../tasks/19-regexp.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn lookbehind_polarity_is_a_private_non_capability_domain() {
    let declaration = bounded(
        REGEXP_SOURCE,
        "enum LookbehindPolarity {",
        "impl LookbehindPolarity {",
    );
    assert_eq!(compact(declaration), "Positive,Negative,}");
    let prefix = bounded(
        REGEXP_SOURCE,
        "enum ParsedTermAtom {",
        "enum LookbehindPolarity {",
    );
    assert!(!prefix.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!REGEXP_SOURCE.contains(&format!("impl {capability} for LookbehindPolarity")));
    }
    assert!(!REGEXP_SOURCE.contains("pub enum LookbehindPolarity"));
    assert!(!REGEXP_SOURCE.contains("pub(crate) enum LookbehindPolarity"));
}

#[test]
fn syntax_and_wire_projection_each_have_one_exhaustive_owner() {
    let implementation = bounded(
        REGEXP_SOURCE,
        "impl LookbehindPolarity {",
        "enum ParsedAtom {",
    );
    let implementation = compact(implementation);
    assert!(implementation.contains(
        "fnfrom_syntax_marker(marker:u8)->Option<Self>{matchmarker{b'='=>Some(Self::Positive),b'!'=>Some(Self::Negative),_=>None,}}"
    ));
    assert!(implementation.contains(
        "constfnoperand_bit(&self)->u64{matchself{Self::Positive=>0,Self::Negative=>1,}}"
    ));
    assert_eq!(REGEXP_SOURCE.matches("from_syntax_marker").count(), 2);
    assert_eq!(REGEXP_SOURCE.matches(".operand_bit()").count(), 2);

    let constructors = bounded(
        REGEXP_SOURCE,
        "const fn lookbehind_end(",
        "pub const fn positive_ascii_class_contains(",
    );
    assert_eq!(
        constructors
            .matches("polarity: &LookbehindPolarity")
            .count(),
        2
    );
    assert_eq!(constructors.matches("polarity.operand_bit()").count(), 2);
    assert!(!constructors.contains("negative: bool"));
    assert!(!constructors.contains("negative as u64"));
}

#[test]
fn parsed_atom_owns_polarity_and_lowering_only_borrows_it() {
    let parsed_atom = bounded(REGEXP_SOURCE, "enum ParsedAtom {", "struct NamedCapture {");
    assert!(parsed_atom.contains("Lookbehind {\n        polarity: LookbehindPolarity,"));
    assert!(!parsed_atom.contains("negative: bool"));

    let parser = bounded(
        REGEXP_SOURCE,
        "Some(b'<') => {",
        "                    Some(b'i' | b'm' | b's' | b'-') => {",
    );
    assert_eq!(
        parser
            .matches("LookbehindPolarity::from_syntax_marker")
            .count(),
        1
    );
    assert_eq!(
        parser
            .matches("ParsedAtom::Lookbehind { polarity, body }")
            .count(),
        1
    );
    assert!(!parser.contains("negative"));

    let lowerer = bounded(
        REGEXP_SOURCE,
        "ParsedAtom::Lookbehind { polarity, body } => {",
        "ParsedAtom::RequiresUnicodeSetSemantics(_) => Ok(()),",
    );
    assert_eq!(lowerer.matches("lookbehind_end(").count(), 2);
    assert_eq!(lowerer.matches("lookbehind_failure(").count(), 2);
    assert_eq!(lowerer.matches(", polarity)").count(), 4);
    assert!(!lowerer.contains("operand_bit"));
    assert!(!lowerer.contains("*polarity"));
    assert!(!lowerer.contains("negative"));
}

#[test]
fn focused_witness_and_evidence_cover_both_polarities() {
    for source in ["(?<=a)b", "(?<!a)b"] {
        assert!(BEHAVIOR_SOURCE.contains(source));
    }
    assert!(BEHAVIOR_SOURCE.contains("assert_eq!(polarity_bits(&positive), (0, 0));"));
    assert!(BEHAVIOR_SOURCE.contains("assert_eq!(polarity_bits(&negative), (1, 1));"));

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("LookbehindPolarity"));
        assert!(evidence.contains("from_syntax_marker"));
        assert!(evidence.contains("operand_bit"));
    }
    assert!(TASK.contains("regexp-lookbehind-polarity.md"));
}

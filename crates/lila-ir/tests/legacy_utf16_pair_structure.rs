const REGEXP_SOURCE: &str = include_str!("../src/regexp.rs");
const LEGACY_PAIR_SOURCE: &str = include_str!("../src/regexp/legacy_utf16_pair.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn legacy_utf16_pair_has_one_private_child_owner() {
    assert_eq!(
        REGEXP_SOURCE.matches("\nmod legacy_utf16_pair;\n").count(),
        1
    );
    assert!(!REGEXP_SOURCE.contains("\npub mod legacy_utf16_pair;\n"));
    assert!(!REGEXP_SOURCE.contains("\npub(crate) mod legacy_utf16_pair;\n"));
    assert!(!REGEXP_SOURCE.contains("\nmod legacy_utf16_pair {\n"));
    assert!(LEGACY_PAIR_SOURCE.starts_with("use super::RegExpInstruction;\n\n"));

    assert_eq!(
        LEGACY_PAIR_SOURCE
            .matches("pub(super) struct LegacyUtf16Pair {")
            .count(),
        1
    );
    assert!(!REGEXP_SOURCE.contains("struct LegacyUtf16Pair {"));
    assert_eq!(
        LEGACY_PAIR_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with("struct ") || line.starts_with("pub(super) struct "))
            .count(),
        1
    );
    for private_field in ["    lead: u32,", "    trail: u32,"] {
        assert_eq!(LEGACY_PAIR_SOURCE.matches(private_field).count(), 1);
    }
    assert!(!LEGACY_PAIR_SOURCE.contains("pub lead:"));
    assert!(!LEGACY_PAIR_SOURCE.contains("pub trail:"));

    for definition in [
        "    pub(super) fn from_scalar(",
        "    pub(super) fn lead_instruction(",
        "    pub(super) fn trail_instruction(",
    ] {
        assert_eq!(
            LEGACY_PAIR_SOURCE.matches(definition).count(),
            1,
            "child must own exactly one `{definition}`"
        );
        assert!(
            !REGEXP_SOURCE.contains(definition),
            "parent must not retain `{definition}`"
        );
    }
    assert_eq!(LEGACY_PAIR_SOURCE.matches("fn ").count(), 3);
    assert_eq!(
        LEGACY_PAIR_SOURCE
            .lines()
            .filter(|line| line.starts_with("    pub(super) fn "))
            .count(),
        3
    );
    assert!(!LEGACY_PAIR_SOURCE.contains("pub(crate) "));
    assert!(!LEGACY_PAIR_SOURCE.contains("\npub fn "));

    assert_eq!(
        REGEXP_SOURCE
            .matches("use legacy_utf16_pair::LegacyUtf16Pair;")
            .count(),
        1
    );
    assert!(!REGEXP_SOURCE.contains("pub use legacy_utf16_pair::LegacyUtf16Pair;"));
    assert!(!REGEXP_SOURCE.contains("pub(crate) use legacy_utf16_pair::LegacyUtf16Pair;"));
}

#[test]
fn legacy_utf16_pair_constructor_and_projections_have_closed_callers() {
    assert_eq!(
        REGEXP_SOURCE
            .matches("LegacyUtf16Pair::from_scalar(ch)")
            .count(),
        1
    );
    assert_eq!(REGEXP_SOURCE.matches("pair.lead_instruction()").count(), 2);
    assert_eq!(REGEXP_SOURCE.matches("pair.trail_instruction()").count(), 2);

    let parsed_term = bounded(REGEXP_SOURCE, "enum ParsedTerm {", "mod legacy_utf16_pair;");
    assert_eq!(parsed_term.matches("    Quantified {").count(), 1);
    assert_eq!(parsed_term.matches("    LegacyUtf16Pair {").count(), 1);

    let parsed_term_atom = bounded(
        REGEXP_SOURCE,
        "enum ParsedTermAtom {",
        "/// Whether a lookbehind succeeds",
    );
    assert_eq!(
        parsed_term_atom
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != "}")
            .collect::<Vec<_>>(),
        ["Ordinary(ParsedAtom),", "LegacyUtf16Pair(LegacyUtf16Pair),"]
    );

    let parser = bounded(
        REGEXP_SOURCE,
        "fn parse_instruction_atom(",
        "fn regexp_capture_syntax(",
    );
    assert_before(
        parser,
        "let pair = LegacyUtf16Pair::from_scalar(ch)",
        "return Ok(ParsedTermAtom::LegacyUtf16Pair(pair));",
    );
    assert_eq!(
        REGEXP_SOURCE
            .matches("ParsedTerm::LegacyUtf16Pair { .. } => false,")
            .count(),
        2,
        "both nullability projections must keep the mandatory lead non-nullable"
    );
}

#[test]
fn forward_and_reverse_lowering_preserve_surrogate_instruction_order() {
    let forward = bounded(
        REGEXP_SOURCE,
        "    fn sequence(&mut self, terms: &[ParsedTerm])",
        "    fn quantified(",
    );
    assert_before(
        forward,
        "self.push(pair.lead_instruction())?;",
        "&ParsedAtom::Instruction(pair.trail_instruction()),",
    );
    assert_eq!(
        forward
            .matches("self.push(pair.lead_instruction())?;")
            .count(),
        1
    );
    assert_eq!(
        forward
            .matches("&ParsedAtom::Instruction(pair.trail_instruction()),")
            .count(),
        1
    );

    let reverse = bounded(
        REGEXP_SOURCE,
        "    fn reverse_sequence(&mut self, terms: &[ParsedTerm])",
        "    fn reverse_quantified(",
    );
    assert_before(
        reverse,
        "&ParsedAtom::Instruction(pair.trail_instruction()),",
        "self.push(pair.lead_instruction())?;",
    );
    assert_eq!(
        reverse
            .matches("self.push(pair.lead_instruction())?;")
            .count(),
        1
    );
    assert_eq!(
        reverse
            .matches("&ParsedAtom::Instruction(pair.trail_instruction()),")
            .count(),
        1
    );
    assert!(!LEGACY_PAIR_SOURCE.contains("ParsedTerm"));
    assert!(!LEGACY_PAIR_SOURCE.contains("Quantifier"));
}

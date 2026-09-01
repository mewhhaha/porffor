const IR_SOURCE: &str = include_str!("../../lila-ir/src/regexp.rs");
const REGRESS_ROOT_SOURCE: &str = include_str!("../../../vendor/regress-0.10.5/src/lib.rs");
const REGRESS_UNICODE_TABLES_SOURCE: &str =
    include_str!("../../../vendor/regress-0.10.5/src/unicodetables.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

const STRING_PROPERTIES: [(&str, &str, &str); 7] = [
    ("Basic_Emoji", "BasicEmoji", "basic_emoji_sets"),
    (
        "Emoji_Keycap_Sequence",
        "EmojiKeycapSequence",
        "emoji_keycap_sequence_sets",
    ),
    (
        "RGI_Emoji_Flag_Sequence",
        "RGIEmojiFlagSequence",
        "rgi_emoji_flag_sequence_sets",
    ),
    (
        "RGI_Emoji_Modifier_Sequence",
        "RGIEmojiModifierSequence",
        "rgi_emoji_modifier_sequence_sets",
    ),
    (
        "RGI_Emoji_Tag_Sequence",
        "RGIEmojiTagSequence",
        "rgi_emoji_tag_sequence_sets",
    ),
    (
        "RGI_Emoji_ZWJ_Sequence",
        "RGIEmojiZWJSequence",
        "rgi_emoji_zwj_sequence_sets",
    ),
    ("RGI_Emoji", "RGIEmoji", "rgi_emoji_sets"),
];

#[test]
fn provider_exports_one_closed_unicode_string_property_domain() {
    let root_exports = bounded(
        REGRESS_ROOT_SOURCE,
        "pub use crate::unicodetables::{",
        "\n};",
    );
    for exported in [
        "UnicodeStringProperty",
        "unicode_string_property_from_str",
        "unicode_string_property_sequences",
    ] {
        assert!(
            root_exports.contains(exported),
            "missing provider export: {exported}"
        );
    }
    assert!(!REGRESS_ROOT_SOURCE.contains("pub mod unicodetables;"));

    let provider = bounded(
        REGRESS_UNICODE_TABLES_SOURCE,
        "pub enum UnicodeStringProperty {",
        "\npub fn unicode_string_property_from_str",
    );
    for (_, variant, sequence_table) in STRING_PROPERTIES {
        assert_eq!(
            provider.matches(&format!("    {variant},\n")).count(),
            1,
            "provider must declare {variant} exactly once"
        );
        assert_eq!(
            provider
                .matches(&format!("        {variant} => {sequence_table}(),"))
                .count(),
            1,
            "provider must project {variant} exactly once"
        );
    }
    assert_eq!(provider.matches("=>").count(), 7);
    assert!(!provider.contains("_ =>"));

    let parser = bounded(
        REGRESS_UNICODE_TABLES_SOURCE,
        "pub fn unicode_string_property_from_str",
        "\n}",
    );
    for (property_name, variant, _) in STRING_PROPERTIES {
        let arm = format!("        \"{property_name}\" => Some({variant}),");
        assert_eq!(parser.matches(&arm).count(), 1, "missing strict arm: {arm}");
    }
    assert_eq!(parser.matches("=>").count(), 8);
    assert_eq!(parser.matches("_ => None").count(), 1);
}

#[test]
fn lila_projects_all_seven_properties_without_raw_name_matching() {
    let projection = bounded(
        IR_SOURCE,
        "fn parse_unicode_property_of_strings(",
        "\n/// Validates one `ClassStringDisjunction`",
    );
    assert!(projection.contains("unicode_string_property_from_str(value)"));
    for (property_name, variant, _) in STRING_PROPERTIES {
        assert_eq!(
            projection
                .matches(&format!("UnicodeStringProperty::{variant} =>"))
                .count(),
            1,
            "Lila must own {variant} exactly once"
        );
        assert!(
            !projection.contains(property_name),
            "Lila must not duplicate the provider's raw match: {property_name}"
        );
    }
    assert_eq!(projection.matches("UnicodeStringProperty::").count(), 7);
    assert_eq!(projection.matches("=>").count(), 7);
    assert!(!projection.contains("_ =>"));
    assert_eq!(
        projection
            .matches("unsupported_property_of_strings")
            .count(),
        6
    );
}

#[test]
fn only_keycap_consumes_the_provider_sequence_table() {
    let keycap_rows = bounded(
        REGRESS_UNICODE_TABLES_SOURCE,
        "static EMOJI_KEYCAP_SEQUENCE: &[&[u32]; 12] = &[\n",
        "\n];",
    );
    assert_eq!(keycap_rows.matches("    &[").count(), 12);
    for row in [
        "&[35, 65039, 8419]",
        "&[42, 65039, 8419]",
        "&[48, 65039, 8419]",
        "&[57, 65039, 8419]",
    ] {
        assert!(
            keycap_rows.contains(row),
            "provider keycap table lost {row}"
        );
    }

    let projection = bounded(
        IR_SOURCE,
        "fn parse_unicode_property_of_strings(",
        "\n/// Validates one `ClassStringDisjunction`",
    );
    assert_eq!(
        projection
            .matches("unicode_string_property_sequences(property)")
            .count(),
        1
    );
    let keycap_arm = bounded(
        projection,
        "UnicodeStringProperty::EmojiKeycapSequence => {",
        "\n        UnicodeStringProperty::BasicEmoji =>",
    );
    assert!(keycap_arm.contains("unicode_string_property_sequences(property)"));
    assert!(keycap_arm.contains(".map(|sequence| sequence.to_vec())"));
    assert!(keycap_arm.contains("finite_case_invariant_property_of_strings(strings)"));
    assert!(!IR_SOURCE.contains("b\"#*0123456789\""));
}

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

    let declaration = bounded(
        REGRESS_UNICODE_TABLES_SOURCE,
        "macro_rules! unicode_string_property_rows {",
        "\nunicode_string_property_rows! {",
    );
    assert!(declaration.contains("pub enum UnicodeStringProperty { $($variant),+ }"));
    assert!(declaration.contains("pub const ALL: &'static [Self] = &[$(Self::$variant),+];"));
    assert!(declaration
        .contains("match s { $($name => Some(UnicodeStringProperty::$variant),)+ _ => None }"));
    let rows = bounded(
        REGRESS_UNICODE_TABLES_SOURCE,
        "unicode_string_property_rows! {",
        "\n}",
    );
    let sequences = bounded(
        REGRESS_UNICODE_TABLES_SOURCE,
        "pub fn unicode_string_property_sequences(",
        "\n}",
    );
    for (property_name, variant, sequence_table) in STRING_PROPERTIES {
        assert_eq!(
            rows.matches(&format!("{variant} => \"{property_name}\""))
                .count(),
            1,
            "the provider row owns {property_name} exactly once"
        );
        assert_eq!(
            sequences
                .matches(&format!("{variant} => {sequence_table}(),"))
                .count(),
            1,
            "the provider projects {variant} exactly once"
        );
    }
    assert_eq!(rows.matches("=>").count(), 7);
    assert_eq!(sequences.matches("=>").count(), 7);
    assert!(!sequences.contains("_ =>"));
}

#[test]
fn lila_projects_all_seven_properties_without_raw_name_matching() {
    let projection = bounded(
        IR_SOURCE,
        "fn parse_unicode_property_of_strings(",
        "\n/// Validates one `ClassStringDisjunction`",
    );
    assert!(projection.contains("unicode_string_property_from_str(value)"));
    for (property_name, _, _) in STRING_PROPERTIES {
        assert!(
            !projection.contains(property_name),
            "Lila must not duplicate the provider's raw match: {property_name}"
        );
    }
    assert!(projection.contains("regexp_unicode_property_catalog()"));
    assert!(projection.contains("RegExpUnicodePropertyCatalogValue::Strings(sequences)"));
    assert!(projection.contains("sequences.iter().cloned().collect()"));
    assert!(projection.contains("ClassSetValue::finite_property_of_strings(strings, folding)"));
}

#[test]
fn all_properties_consume_the_provider_sequence_table() {
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
    let catalog = bounded(
        IR_SOURCE,
        "pub fn regexp_unicode_property_catalog()",
        "\n/// Resolves the validated property",
    );
    assert!(catalog.contains("for &property in UnicodeStringProperty::ALL"));
    assert!(catalog.contains(
        "UnicodePropertyCatalogValue::Strings(validated_unicode_string_property_sequences("
    ));
    let constructor = bounded(
        IR_SOURCE,
        "fn validated_unicode_string_property_sequences(",
        "\n/// Resolves the validated property",
    );
    assert_eq!(
        constructor
            .matches("unicode_string_property_sequences(property)")
            .count(),
        1
    );
    assert!(catalog.contains("sequence.to_vec()"));
    assert!(projection.contains("regexp_unicode_property_catalog()"));
    assert!(projection.contains("RegExpUnicodePropertyCatalogValue::Strings(sequences)"));
    assert!(projection.contains("ClassSetValue::finite_property_of_strings(strings, folding)"));
    assert!(!IR_SOURCE.contains("b\"#*0123456789\""));
}

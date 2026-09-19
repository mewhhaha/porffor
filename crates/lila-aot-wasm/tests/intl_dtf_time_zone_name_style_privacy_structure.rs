use lila_intl::TimeZoneNameStyle;

const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const DOMAIN_SOURCE: &str = include_str!("../../lila-intl/src/time_zone.rs");

#[test]
fn constructor_and_provider_share_one_closed_style_domain() {
    assert!(!DTF_SOURCE.contains("enum TimeZoneNameStyle"));
    assert!(DTF_SOURCE.contains("&TimeZoneNameStyle::OPTIONS"));
    assert_eq!(
        DOMAIN_SOURCE
            .matches("pub enum TimeZoneNameStyle {")
            .count(),
        1
    );
    assert_eq!(TimeZoneNameStyle::ALL.len(), 6);
    for (style, (spelling, code)) in TimeZoneNameStyle::ALL
        .into_iter()
        .zip(TimeZoneNameStyle::OPTIONS)
    {
        assert_eq!(style.spelling(), spelling);
        assert_eq!(style.code(), code);
        assert_eq!(TimeZoneNameStyle::from_code(code), Some(style));
    }
    assert_eq!(TimeZoneNameStyle::from_code(0), None);
    assert_eq!(TimeZoneNameStyle::from_code(7), None);
}

#[test]
fn shared_style_spelling_projection_is_exhaustive() {
    let projection = DOMAIN_SOURCE
        .split_once("pub const fn spelling(self)")
        .unwrap()
        .1
        .split_once("#[derive")
        .unwrap()
        .0;
    assert!(!projection.contains("_ =>"));
    for style in TimeZoneNameStyle::ALL {
        assert!(projection.contains(style.spelling()));
    }
    assert!(!DTF_SOURCE.contains("emit_dtf_time_zone_name_value"));
    assert!(!DTF_SOURCE.contains("INTL_DTF_GMT_PREFIX"));
}

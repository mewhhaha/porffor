use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Profile {
    pub(super) schema_version: u32,
    pub(super) selector: Selector,
    pub(super) numbering_systems: Vec<Numbering>,
    pub(super) algorithmic_fields: Vec<Algorithmic>,
    pub(super) zone_geography: Geography,
    pub(super) locales: Vec<Locale>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Selector {
    pub(super) schema_version: u32,
    pub(super) locales: Vec<String>,
    pub(super) calendars: Vec<String>,
    pub(super) default_locale: String,
    pub(super) minimum_draft: String,
    pub(super) alt_selection: String,
    pub(super) calendar_identifiers: BTreeMap<String, String>,
    pub(super) release: String,
    pub(super) commit: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Numbering {
    pub(super) identifier: String,
    pub(super) digits: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Algorithmic {
    pub(super) identifier: String,
    pub(super) field: char,
    pub(super) minimum: u8,
    pub(super) values: Vec<String>,
    pub(super) source: String,
    pub(super) ruleset: String,
    pub(super) consumed_rules: Vec<(String, u64, String)>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Locale {
    pub(super) locale: String,
    pub(super) territory: String,
    pub(super) parent_chain: Vec<String>,
    pub(super) default_content: bool,
    pub(super) default_numbering: String,
    pub(super) decimal_separators: Vec<(String, String)>,
    pub(super) minus_signs: Vec<(String, String)>,
    pub(super) calendar_preferences: Vec<String>,
    pub(super) preferred_hour: char,
    pub(super) allowed_hours: Vec<String>,
    pub(super) day_period_rules: Vec<PeriodRule>,
    pub(super) calendars: BTreeMap<String, Calendar>,
    pub(super) zone_names: ZoneNames,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PeriodRule {
    #[serde(rename = "type")]
    pub(super) period: String,
    pub(super) at: Option<String>,
    pub(super) from: Option<String>,
    pub(super) before: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Calendar {
    pub(super) calendar: String,
    pub(super) names: Vec<Name>,
    pub(super) styles: BTreeMap<String, Style>,
    pub(super) available: Vec<Available>,
    pub(super) intervals: Vec<Interval>,
    pub(super) interval_fallback: Pattern,
    pub(super) append_zone: Pattern,
    pub(super) append_era: Option<Pattern>,
    pub(super) excluded_non_ecma_formats: Vec<Excluded>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Name {
    pub(super) kind: String,
    pub(super) context: Option<String>,
    pub(super) width: Option<String>,
    pub(super) index: Option<u8>,
    pub(super) period: Option<String>,
    pub(super) value: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Style {
    pub(super) date: Pattern,
    pub(super) time: Pattern,
    pub(super) standard: Pattern,
    #[serde(rename = "atTime")]
    pub(super) at_time: Pattern,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Pattern {
    pub(super) source: String,
    pub(super) tokens: Vec<Token>,
    pub(super) numbering_overrides: Vec<NumberingOverride>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Available {
    pub(super) skeleton: String,
    pub(super) source: String,
    pub(super) tokens: Vec<Token>,
    pub(super) numbering_overrides: Vec<NumberingOverride>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NumberingOverride {
    pub(super) field: Option<char>,
    pub(super) numbering: String,
}
#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum Token {
    Literal(LiteralToken),
    Field(FieldToken),
    Placeholder(PlaceholderToken),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LiteralToken {
    pub(super) literal: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FieldToken {
    pub(super) field: char,
    pub(super) width: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PlaceholderToken {
    pub(super) placeholder: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Interval {
    pub(super) skeleton: String,
    pub(super) greatest_difference: char,
    pub(super) source: String,
    pub(super) tokens: Vec<Token>,
    pub(super) numbering_overrides: Vec<NumberingOverride>,
    pub(super) second_start: usize,
    pub(super) shared_fields: Vec<String>,
    pub(super) endpoint_order: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Excluded {
    pub(super) skeleton: String,
    pub(super) path: String,
    pub(super) reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Geography {
    pub(super) aliases: Vec<(String, String)>,
    pub(super) zones: Vec<Zone>,
    pub(super) metazones: Vec<Metazone>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Zone {
    pub(super) identifier: String,
    pub(super) territory: String,
    pub(super) periods: Vec<(i64, i64, String)>,
    pub(super) location_is_country: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Metazone {
    pub(super) identifier: String,
    pub(super) preferred: Vec<(String, String)>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ZoneNames {
    pub(super) patterns: BTreeMap<String, String>,
    pub(super) zones: Vec<ZoneName>,
    pub(super) metazones: Vec<MetazoneName>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ZoneName {
    pub(super) identifier: String,
    pub(super) city: String,
    pub(super) country: Option<String>,
    pub(super) location: Option<String>,
    pub(super) names: WidthNames,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MetazoneName {
    pub(super) identifier: String,
    pub(super) names: WidthNames,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WidthNames {
    pub(super) short: NameVariants,
    pub(super) long: NameVariants,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NameVariants {
    pub(super) generic: Option<String>,
    pub(super) standard: Option<String>,
    pub(super) daylight: Option<String>,
}

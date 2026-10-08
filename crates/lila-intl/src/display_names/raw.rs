//! Raw data is confined to admission. Consumers never accept these structs.
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Profile {
    pub schema_version: u32,
    pub cldr_release: String,
    pub cldr_commit: String,
    pub source_manifest_sha256: String,
    pub bcp47_manifest_sha256: String,
    pub default_locale: String,
    pub name_pool: Vec<NamePool>,
    pub locales: Vec<Locale>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Locale {
    pub locale: String,
    pub locale_pattern: String,
    pub locale_separator: String,
    pub styles: Styles,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Styles {
    pub long: Names,
    pub short: Names,
    pub narrow: Names,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Names {
    pub language: u32,
    pub region: u32,
    pub script: u32,
    pub language_script: u32,
    pub currency: u32,
    pub calendar: u32,
    pub date_time_field: u32,
    pub variant: u32,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Domain {
    Language,
    Region,
    Script,
    LanguageScript,
    Currency,
    Calendar,
    DateTimeField,
    Variant,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NamePool {
    pub kind: Domain,
    pub entries: Vec<(String, String)>,
}

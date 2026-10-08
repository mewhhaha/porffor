use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Profile {
    pub schema: u32,
    pub cldr_commit: String,
    pub default_locale: String,
    pub locales: Vec<Locale>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Locale {
    pub locale: String,
    pub units: Vec<Unit>,
    pub lists: Vec<List>,
    pub digital: Digital,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Unit {
    pub unit: String,
    pub style: String,
    pub patterns: [String; 6],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct List {
    pub style: String,
    pub patterns: [String; 4],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Digital {
    pub hm: Pattern,
    pub hms: Pattern,
    pub ms: Pattern,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Pattern {
    pub pattern: String,
    pub widths: Vec<u8>,
    pub separators: Vec<String>,
}

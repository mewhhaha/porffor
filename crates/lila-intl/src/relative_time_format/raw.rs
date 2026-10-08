use super::{RelativeStyle, RelativeUnit};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Profile {
    pub schema: u8,
    pub cldr_commit: String,
    pub locales: Vec<Locale>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Locale {
    pub locale: String,
    pub fields: Vec<Field>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Field {
    pub unit: RelativeUnit,
    pub style: RelativeStyle,
    pub past: [String; 6],
    pub future: [String; 6],
    pub relative: Vec<Relative>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Relative {
    pub offset: i8,
    pub value: String,
}

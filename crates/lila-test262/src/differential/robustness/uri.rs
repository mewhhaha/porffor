//! Raw UTF-16 URI data reaches one original builtin through an escaped literal.
use super::BuiltinParserTarget;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u32,
    units: Vec<u16>,
}
pub(super) struct UriInput {
    units: Vec<u16>,
}
impl UriInput {
    pub(super) fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > 64 * 1024 {
            return Err("URI input exceeds 64 KiB".into());
        }
        let wire: Wire = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if wire.schema_version != 1 || wire.units.len() > 4096 {
            return Err("URI schema 1 requires at most 4096 UTF-16 units".into());
        }
        Ok(Self { units: wire.units })
    }
    pub(super) fn source(&self, parser: BuiltinParserTarget) -> Result<String, String> {
        let function = match parser {
            BuiltinParserTarget::EncodeUri => "encodeURI",
            BuiltinParserTarget::EncodeUriComponent => "encodeURIComponent",
            BuiltinParserTarget::DecodeUri => "decodeURI",
            BuiltinParserTarget::DecodeUriComponent => "decodeURIComponent",
            BuiltinParserTarget::Json
            | BuiltinParserTarget::RegExp
            | BuiltinParserTarget::Number
            | BuiltinParserTarget::BigInt
            | BuiltinParserTarget::TemporalDate => {
                return Err("URI input requires its closed URI operation".into())
            }
        };
        let mut source = format!("{function}(\"");
        for unit in &self.units {
            use std::fmt::Write;
            write!(source, "\\u{unit:04x}").expect("String formatting");
        }
        source.push_str("\");");
        Ok(source)
    }
}

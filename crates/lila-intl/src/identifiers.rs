use core::fmt;

/// Maximum byte length accepted by the current Intl identifier wire domain.
///
/// Locale identifiers are ASCII, so this is both the UTF-8 byte limit and the
/// maximum result capacity an emitter must reserve for canonicalization.
pub const MAX_INTL_IDENTIFIER_BYTES: usize = 255;

/// A structurally checked locale identifier observed by the Wasm shell.
///
/// This is deliberately distinct from [`CanonicalLocaleId`]. Structural
/// validation happens before the provider boundary; alias resolution and
/// canonical spelling happen behind it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocaleId(Box<str>);

impl LocaleId {
    pub fn parse(raw: impl Into<Box<str>>) -> Result<Self, InvalidLocaleId> {
        let raw = raw.into();
        if valid_locale_syntax(&raw) {
            Ok(Self(raw))
        } else {
            Err(InvalidLocaleId { value: raw })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A locale identifier after provider-backed alias resolution and canonical
/// ASCII casing.
///
/// `from_data` is the generated-data boundary: it verifies protocol spelling,
/// while the provider generator remains responsible for resolving the pinned
/// CLDR alias tables before calling it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalLocaleId(Box<str>);

impl CanonicalLocaleId {
    pub fn from_data(raw: impl Into<Box<str>>) -> Result<Self, InvalidCanonicalLocaleId> {
        let raw = raw.into();
        if valid_locale_syntax(&raw) && has_canonical_locale_case(&raw) {
            Ok(Self(raw))
        } else {
            Err(InvalidCanonicalLocaleId { value: raw })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A structurally checked time-zone identifier observed by the Wasm shell.
///
/// Its spelling is not assumed to be primary or canonical. In particular,
/// IANA link names and ASCII case variants remain valid provider inputs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeZoneId(Box<str>);

impl TimeZoneId {
    pub fn parse(raw: impl Into<Box<str>>) -> Result<Self, InvalidTimeZoneId> {
        let raw = raw.into();
        if valid_time_zone_syntax(&raw) {
            Ok(Self(raw))
        } else {
            Err(InvalidTimeZoneId { value: raw })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A primary time-zone identifier selected from the pinned shared catalogue.
///
/// `from_data` validates the binary-protocol spelling. The generated catalogue
/// is responsible for resolving links and case-insensitive lookups before it
/// constructs this type. UTC aliases must be normalized to `UTC`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalTimeZoneId(Box<str>);

impl CanonicalTimeZoneId {
    pub fn from_data(raw: impl Into<Box<str>>) -> Result<Self, InvalidCanonicalTimeZoneId> {
        let raw = raw.into();
        let utc_alias = raw.eq_ignore_ascii_case("UTC")
            || raw.eq_ignore_ascii_case("GMT")
            || raw.eq_ignore_ascii_case("Etc/GMT")
            || raw.eq_ignore_ascii_case("Etc/UTC");
        let canonical_utc = !utc_alias || &*raw == "UTC";
        if valid_time_zone_syntax(&raw) && canonical_utc {
            Ok(Self(raw))
        } else {
            Err(InvalidCanonicalTimeZoneId { value: raw })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_locale_syntax(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_INTL_IDENTIFIER_BYTES || !raw.is_ascii() {
        return false;
    }

    let mut subtags = raw.split('-').peekable();
    let Some(language) = subtags.next() else {
        return false;
    };
    let language_valid = ((2..=8).contains(&language.len())
        && language.bytes().all(|byte| byte.is_ascii_alphabetic()))
        || (language.len() == 1 && language.eq_ignore_ascii_case("i"))
        || (language.len() == 1 && language.eq_ignore_ascii_case("x"));
    if !language_valid {
        return false;
    }
    if language.len() == 1 && subtags.peek().is_none() {
        return false;
    }

    let mut private_use = language.eq_ignore_ascii_case("x");
    while let Some(subtag) = subtags.next() {
        if subtag.is_empty()
            || subtag.len() > 8
            || !subtag.bytes().all(|byte| byte.is_ascii_alphanumeric())
        {
            return false;
        }
        if subtag.len() == 1 && !private_use && subtags.peek().is_none() {
            return false;
        }
        private_use |= subtag.eq_ignore_ascii_case("x");
    }
    true
}

fn has_canonical_locale_case(raw: &str) -> bool {
    let mut subtags = raw.split('-');
    let Some(language) = subtags.next() else {
        return false;
    };
    if !is_ascii_lowercase_or_digit(language) {
        return false;
    }

    let mut extension = language.len() == 1;
    let mut script_seen = false;
    let mut region_seen = false;
    for subtag in subtags {
        if subtag.len() == 1 {
            extension = true;
            if !is_ascii_lowercase_or_digit(subtag) {
                return false;
            }
            continue;
        }
        if extension {
            if !is_ascii_lowercase_or_digit(subtag) {
                return false;
            }
        } else if !script_seen
            && subtag.len() == 4
            && subtag.bytes().all(|byte| byte.is_ascii_alphabetic())
        {
            script_seen = true;
            let mut bytes = subtag.bytes();
            if !bytes.next().is_some_and(|byte| byte.is_ascii_uppercase())
                || !bytes.all(|byte| byte.is_ascii_lowercase())
            {
                return false;
            }
        } else if !region_seen
            && ((subtag.len() == 2 && subtag.bytes().all(|byte| byte.is_ascii_alphabetic()))
                || (subtag.len() == 3 && subtag.bytes().all(|byte| byte.is_ascii_digit())))
        {
            region_seen = true;
            if !subtag
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
            {
                return false;
            }
        } else if !is_ascii_lowercase_or_digit(subtag) {
            return false;
        }
    }
    true
}

fn is_ascii_lowercase_or_digit(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn valid_time_zone_syntax(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_INTL_IDENTIFIER_BYTES || !raw.is_ascii() {
        return false;
    }
    if matches!(bytes[0], b'+' | b'-') {
        return valid_fixed_offset(bytes);
    }
    raw.split('/').all(|component| {
        !component.is_empty()
            && component != "."
            && component != ".."
            && component.len() <= 64
            && component
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+'))
    })
}

fn valid_fixed_offset(bytes: &[u8]) -> bool {
    if bytes.len() != 6 || bytes[3] != b':' {
        return false;
    }
    let digits = [bytes[1], bytes[2], bytes[4], bytes[5]];
    if !digits.iter().all(u8::is_ascii_digit) {
        return false;
    }
    let hours = (bytes[1] - b'0') * 10 + bytes[2] - b'0';
    let minutes = (bytes[4] - b'0') * 10 + bytes[5] - b'0';
    hours <= 23 && minutes <= 59
}

macro_rules! invalid_identifier_error {
    ($name:ident, $message:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            value: Box<str>,
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!($message, " {:?}"), self.value)
            }
        }

        impl std::error::Error for $name {}
    };
}

invalid_identifier_error!(InvalidLocaleId, "invalid locale identifier");
invalid_identifier_error!(
    InvalidCanonicalLocaleId,
    "invalid canonical locale identifier"
);
invalid_identifier_error!(InvalidTimeZoneId, "invalid time-zone identifier");
invalid_identifier_error!(
    InvalidCanonicalTimeZoneId,
    "invalid canonical time-zone identifier"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_inputs_and_provider_outputs_are_distinct() {
        assert!(LocaleId::parse("EN-us-u-CA-gregory").is_ok());
        assert!(CanonicalLocaleId::from_data("en-US-u-ca-gregory").is_ok());
        assert!(CanonicalLocaleId::from_data("EN-us").is_err());
        assert!(LocaleId::parse("en--US").is_err());
    }

    #[test]
    fn time_zone_inputs_accept_links_but_outputs_normalize_utc() {
        assert!(TimeZoneId::parse("europe/stockholm").is_ok());
        assert!(CanonicalTimeZoneId::from_data("Europe/Stockholm").is_ok());
        assert!(TimeZoneId::parse("Etc/UTC").is_ok());
        assert!(CanonicalTimeZoneId::from_data("Etc/UTC").is_err());
        assert!(CanonicalTimeZoneId::from_data("UTC").is_ok());
    }

    #[test]
    fn fixed_offset_syntax_has_a_bounded_domain() {
        assert!(TimeZoneId::parse("+23:59").is_ok());
        assert!(TimeZoneId::parse("+24:00").is_err());
        assert!(TimeZoneId::parse("+0100").is_err());
    }
}

//! Framed Locale information requests and checked native name-list responses.
use crate::{
    CanonicalLocaleId, DateTimeCalendar, IntlOperation, LocaleCalendars, LocaleCalendarsOperation,
    LocaleCollations, LocaleCollationsOperation, LocaleTimeZones, LocaleTimeZonesOperation,
};
use core::fmt;

pub const LOCALE_INFORMATION_WIRE_VERSION: u64 = 1;
pub const LOCALE_INFORMATION_HEADER_BYTES: u64 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleInformationWireError {
    Malformed(&'static str),
    Resource(&'static str),
}

impl fmt::Display for LocaleInformationWireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(reason) | Self::Resource(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for LocaleInformationWireError {}

/// Only the semantic owners' private, validated results can enter this codec.
/// The operation tag follows the variant; callers cannot substitute another tag.
pub enum LocaleInformationResponse<'a> {
    Calendars(&'a LocaleCalendars),
    Collations(&'a LocaleCollations),
    TimeZones(&'a LocaleTimeZones),
}

#[derive(Clone, Copy)]
enum NameKind {
    Calendar,
    Collation,
    TimeZone,
}

impl NameKind {
    fn operation(self) -> u64 {
        match self {
            Self::Calendar => u64::from(LocaleCalendarsOperation::HOST_OP.code()),
            Self::Collation => u64::from(LocaleCollationsOperation::HOST_OP.code()),
            Self::TimeZone => u64::from(LocaleTimeZonesOperation::HOST_OP.code()),
        }
    }

    fn valid(self, name: &str) -> bool {
        match self {
            Self::Calendar => {
                DateTimeCalendar::parse(name).is_some_and(|calendar| calendar.as_str() == name)
            }
            Self::Collation => {
                !matches!(name, "standard" | "search" | "searchjl")
                    && name.split('-').all(|subtag| {
                        (3..=8).contains(&subtag.len())
                            && subtag
                                .bytes()
                                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                    })
            }
            Self::TimeZone => {
                !name.is_empty()
                    && name.split('/').all(|part| !part.is_empty())
                    && name.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-' | b'+')
                    })
            }
        }
    }
}

fn checked_extent(current: usize, additional: usize) -> Result<usize, LocaleInformationWireError> {
    let length = current
        .checked_add(additional)
        .ok_or(LocaleInformationWireError::Resource("wire size overflow"))?;
    u32::try_from(length)
        .map_err(|_| LocaleInformationWireError::Resource("wire exceeds Wasm32 span"))?;
    Ok(length)
}

fn append(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), LocaleInformationWireError> {
    checked_extent(output.len(), bytes.len())?;
    output
        .try_reserve(bytes.len())
        .map_err(|_| LocaleInformationWireError::Resource("wire allocation failed"))?;
    output.extend_from_slice(bytes);
    Ok(())
}

fn encode_names(kind: NameKind, names: &[Box<str>]) -> Result<Vec<u8>, LocaleInformationWireError> {
    if matches!(kind, NameKind::Calendar) && names.is_empty() {
        return Err(LocaleInformationWireError::Malformed(
            "empty calendar preference list",
        ));
    }
    let mut output = Vec::new();
    for word in [
        LOCALE_INFORMATION_WIRE_VERSION,
        kind.operation() * 2 + 1,
        u64::try_from(names.len())
            .map_err(|_| LocaleInformationWireError::Resource("name count overflow"))?,
    ] {
        append(&mut output, &word.to_le_bytes())?;
    }
    for name in names {
        if !kind.valid(name) {
            return Err(LocaleInformationWireError::Malformed(
                "invalid default identifier",
            ));
        }
        let length = u64::try_from(name.len())
            .map_err(|_| LocaleInformationWireError::Resource("name length overflow"))?;
        append(&mut output, &length.to_le_bytes())?;
        append(&mut output, name.as_bytes())?;
    }
    Ok(output)
}

impl LocaleInformationResponse<'_> {
    /// Preserve the semantic owner's list order and write exactly its used bytes.
    pub fn encode(self) -> Result<Vec<u8>, LocaleInformationWireError> {
        match self {
            Self::Calendars(result) => encode_names(NameKind::Calendar, result.names()),
            Self::Collations(result) => encode_names(NameKind::Collation, result.names()),
            Self::TimeZones(result) => encode_names(NameKind::TimeZone, result.names()),
        }
    }
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], LocaleInformationWireError> {
        let (field, remaining) =
            self.0
                .split_at_checked(length)
                .ok_or(LocaleInformationWireError::Malformed(
                    "truncated wire field",
                ))?;
        self.0 = remaining;
        Ok(field)
    }

    fn word(&mut self) -> Result<u64, LocaleInformationWireError> {
        let bytes: [u8; 8] = self.take(8)?.try_into().expect("checked eight-byte field");
        Ok(u64::from_le_bytes(bytes))
    }
}

fn decode_request(
    kind: NameKind,
    bytes: &[u8],
) -> Result<CanonicalLocaleId, LocaleInformationWireError> {
    checked_extent(0, bytes.len())?;
    let mut reader = Reader(bytes);
    if reader.word()? != LOCALE_INFORMATION_WIRE_VERSION || reader.word()? != kind.operation() * 2 {
        return Err(LocaleInformationWireError::Malformed(
            "wrong request version or operation",
        ));
    }
    let length = u32::try_from(reader.word()?)
        .map_err(|_| LocaleInformationWireError::Malformed("locale exceeds Wasm32 span"))?;
    let text = core::str::from_utf8(reader.take(length as usize)?)
        .map_err(|_| LocaleInformationWireError::Malformed("locale is not UTF-8"))?;
    if !reader.0.is_empty() {
        return Err(LocaleInformationWireError::Malformed(
            "trailing request bytes",
        ));
    }
    CanonicalLocaleId::from_data(text)
        .map_err(|_| LocaleInformationWireError::Malformed("locale is not canonical"))
}

pub fn decode_locale_calendars_request(
    bytes: &[u8],
) -> Result<CanonicalLocaleId, LocaleInformationWireError> {
    decode_request(NameKind::Calendar, bytes)
}

pub fn decode_locale_collations_request(
    bytes: &[u8],
) -> Result<CanonicalLocaleId, LocaleInformationWireError> {
    decode_request(NameKind::Collation, bytes)
}

pub fn decode_locale_time_zones_request(
    bytes: &[u8],
) -> Result<CanonicalLocaleId, LocaleInformationWireError> {
    decode_request(NameKind::TimeZone, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(kind: NameKind, locale: &[u8]) -> Vec<u8> {
        [
            LOCALE_INFORMATION_WIRE_VERSION.to_le_bytes().as_slice(),
            (kind.operation() * 2).to_le_bytes().as_slice(),
            (locale.len() as u64).to_le_bytes().as_slice(),
            locale,
        ]
        .concat()
    }

    #[test]
    fn request_operation_is_closed_and_canonical_locale_is_consumed() {
        for kind in [NameKind::Calendar, NameKind::Collation, NameKind::TimeZone] {
            let bytes = request(kind, b"en-US-u-rg-gbzzzz");
            assert_eq!(
                decode_request(kind, &bytes).unwrap().as_str(),
                "en-US-u-rg-gbzzzz"
            );
            for wrong in [NameKind::Calendar, NameKind::Collation, NameKind::TimeZone] {
                if kind.operation() != wrong.operation() {
                    assert!(decode_request(wrong, &bytes).is_err());
                }
            }
        }
    }

    #[test]
    fn requests_reject_truncation_trailing_bytes_and_unrepresentable_lengths() {
        let bytes = request(NameKind::Calendar, b"en-US");
        for length in 0..bytes.len() {
            assert!(decode_locale_calendars_request(&bytes[..length]).is_err());
        }
        let mut malformed = bytes.clone();
        malformed.push(0);
        assert!(decode_locale_calendars_request(&malformed).is_err());
        malformed = bytes;
        malformed[16..24].copy_from_slice(&(u64::from(u32::MAX) + 1).to_le_bytes());
        assert!(decode_locale_calendars_request(&malformed).is_err());
    }

    #[test]
    fn requests_reject_wrong_versions_non_utf8_and_noncanonical_tags() {
        let mut bytes = request(NameKind::Collation, b"en-US");
        bytes[..8].copy_from_slice(&2u64.to_le_bytes());
        assert!(decode_locale_collations_request(&bytes).is_err());
        for invalid in [b"EN-us".as_slice(), b"en_Us", b"", b"en-\xff"] {
            assert!(
                decode_locale_collations_request(&request(NameKind::Collation, invalid)).is_err()
            );
        }
    }

    #[test]
    fn variable_names_preserve_calendar_preference_order_and_exact_extent() {
        let names = [Box::<str>::from("buddhist"), Box::<str>::from("gregory")];
        let bytes = encode_names(NameKind::Calendar, &names).unwrap();
        assert_eq!(&bytes[..8], &1u64.to_le_bytes());
        assert_eq!(
            &bytes[8..16],
            &(NameKind::Calendar.operation() * 2 + 1).to_le_bytes()
        );
        assert_eq!(&bytes[16..24], &2u64.to_le_bytes());
        assert_eq!(&bytes[24..32], &8u64.to_le_bytes());
        assert_eq!(&bytes[32..40], b"buddhist");
        assert_eq!(&bytes[40..48], &7u64.to_le_bytes());
        assert_eq!(&bytes[48..], b"gregory");
    }

    #[test]
    fn typed_native_responses_bind_their_own_operations_and_empty_country_list() {
        use crate::{
            EmbeddedIntlProvider, IntlOperationProvider, LocaleCalendarsRequest,
            LocaleCollationsRequest, LocaleTimeZonesRequest,
        };
        let provider = EmbeddedIntlProvider::new().unwrap();
        let calendars =
            <EmbeddedIntlProvider as IntlOperationProvider<LocaleCalendarsOperation>>::execute(
                &provider,
                LocaleCalendarsRequest::new(CanonicalLocaleId::from_data("th-TH").unwrap())
                    .unwrap(),
            )
            .unwrap();
        let collations =
            <EmbeddedIntlProvider as IntlOperationProvider<LocaleCollationsOperation>>::execute(
                &provider,
                LocaleCollationsRequest::new(CanonicalLocaleId::from_data("de").unwrap()).unwrap(),
            )
            .unwrap();
        let time_zones = <EmbeddedIntlProvider as IntlOperationProvider<
            LocaleTimeZonesOperation,
        >>::execute(
            &provider,
            LocaleTimeZonesRequest::new(CanonicalLocaleId::from_data("en-001").unwrap()).unwrap(),
        )
        .unwrap();
        for (response, operation, names) in [
            (
                LocaleInformationResponse::Calendars(&calendars),
                LocaleCalendarsOperation::HOST_OP,
                calendars.names(),
            ),
            (
                LocaleInformationResponse::Collations(&collations),
                LocaleCollationsOperation::HOST_OP,
                collations.names(),
            ),
            (
                LocaleInformationResponse::TimeZones(&time_zones),
                LocaleTimeZonesOperation::HOST_OP,
                time_zones.names(),
            ),
        ] {
            let encoded = response.encode().unwrap();
            assert_eq!(
                &encoded[8..16],
                &(u64::from(operation.code()) * 2 + 1).to_le_bytes()
            );
            assert_eq!(&encoded[16..24], &(names.len() as u64).to_le_bytes());
            let mut reader = Reader(&encoded[24..]);
            for expected in names {
                let length = usize::try_from(reader.word().unwrap()).unwrap();
                assert_eq!(reader.take(length).unwrap(), expected.as_bytes());
            }
            assert!(reader.0.is_empty());
        }
        assert!(time_zones.names().is_empty());
        assert_eq!(
            LocaleInformationResponse::TimeZones(&time_zones)
                .encode()
                .unwrap()
                .len(),
            24,
        );
    }

    #[test]
    fn empty_default_lists_are_arrays_and_identifiers_keep_their_own_domains() {
        assert!(encode_names(NameKind::Calendar, &[]).is_err());
        for kind in [NameKind::Collation, NameKind::TimeZone] {
            let bytes = encode_names(kind, &[]).unwrap();
            assert_eq!(bytes.len(), 24);
            assert_eq!(&bytes[16..24], &0u64.to_le_bytes());
        }
        for (kind, name) in [
            (NameKind::Calendar, "unknown"),
            (NameKind::Collation, "search"),
            (NameKind::Collation, "PHONEBK"),
            (NameKind::TimeZone, "America//New_York"),
            (NameKind::TimeZone, "Europe/París"),
            (NameKind::TimeZone, "UTC\0"),
        ] {
            assert!(encode_names(kind, &[name.into()]).is_err());
        }
        assert!(encode_names(NameKind::Collation, &["phonebk".into()]).is_ok());
        assert!(encode_names(NameKind::TimeZone, &["America/New_York".into()]).is_ok());
    }

    #[test]
    fn aggregate_wire_extent_cannot_overflow_or_cross_the_host_span_domain() {
        assert_eq!(checked_extent(16, 8).unwrap(), 24);
        assert!(checked_extent(u32::MAX as usize, 1).is_err());
        assert!(checked_extent(usize::MAX, 1).is_err());
    }
}

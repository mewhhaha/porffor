//! Closed host messages for RelativeTimeFormat locale resolution and formatting.

use crate::number_format::NumberingSystemOption;
use crate::number_format::numeric::ObservedNumericInput;
use crate::{
    CanonicalLocaleId, IntlHostOp, IntlServiceError, RelativeTimeFormatOptions,
    RelativeTimeFormatRequest, RelativeTimeLocaleQuery, RelativeTimeLocaleRequest,
    RelativeTimeLocaleResult, RelativeTimeNumberKind, RelativeTimeNumeric, RelativeTimePart,
    RelativeTimePartKind, RelativeTimeParts, RelativeTimeStyle, RelativeTimeUnit,
    ResolvedRelativeTimeFormat, ServiceLocaleMatcher,
};

pub const RELATIVE_TIME_WIRE_VERSION: u64 = 1;
pub const RELATIVE_TIME_WIRE_HEADER_BYTES: u64 = 16;

#[derive(Clone, Copy)]
enum Operation {
    ResolveLocale,
    Format,
}

impl Operation {
    fn host(self) -> IntlHostOp {
        match self {
            Self::ResolveLocale => IntlHostOp::ResolveRelativeTimeLocale,
            Self::Format => IntlHostOp::FormatRelativeTime,
        }
    }

    fn tag(self, response: bool) -> u64 {
        u64::from(self.host().code()) * 2 + u64::from(response)
    }
}

struct Writer(Vec<u8>);

impl Writer {
    fn new(operation: Operation, response: bool) -> Self {
        let mut writer = Self(Vec::new());
        writer.word(RELATIVE_TIME_WIRE_VERSION);
        writer.word(operation.tag(response));
        writer
    }

    fn word(&mut self, value: u64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    fn bytes(&mut self, bytes: &[u8]) -> Result<(), IntlServiceError> {
        let length = u64::try_from(bytes.len()).map_err(|_| IntlServiceError::InvalidWire)?;
        self.word(length);
        self.0
            .try_reserve(bytes.len())
            .map_err(|_| IntlServiceError::InvalidWire)?;
        self.0.extend_from_slice(bytes);
        Ok(())
    }

    fn text(&mut self, value: &str) -> Result<(), IntlServiceError> {
        self.bytes(value.as_bytes())
    }

    fn locale(&mut self, value: &CanonicalLocaleId) -> Result<(), IntlServiceError> {
        self.text(value.as_str())
    }

    fn locales(&mut self, values: &[CanonicalLocaleId]) -> Result<(), IntlServiceError> {
        self.word(u64::try_from(values.len()).map_err(|_| IntlServiceError::InvalidWire)?);
        for value in values {
            self.locale(value)?;
        }
        Ok(())
    }

    fn configuration(
        &mut self,
        value: &ResolvedRelativeTimeFormat,
    ) -> Result<(), IntlServiceError> {
        self.locale(value.locale())?;
        self.locale(value.data_locale())?;
        self.text(value.numbering_system())?;
        self.word(style_code(value.options().style()));
        self.word(numeric_code(value.options().numeric()));
        Ok(())
    }

    fn input(&mut self, value: &ObservedNumericInput) -> Result<(), IntlServiceError> {
        match value {
            ObservedNumericInput::NumberShortestDecimal(value) => {
                self.word(RelativeTimeNumberKind::ShortestDecimal.wire_code());
                self.text(value)
            }
            ObservedNumericInput::NegativeZero => {
                self.word(RelativeTimeNumberKind::NegativeZero.wire_code());
                self.bytes(&[])
            }
            ObservedNumericInput::StringNumericLiteral(_)
            | ObservedNumericInput::BigIntDecimal(_) => Err(IntlServiceError::InvalidWire),
        }
    }

    fn finish(self) -> Vec<u8> {
        self.0
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    fn new(
        bytes: &'a [u8],
        operation: Operation,
        response: bool,
    ) -> Result<Self, IntlServiceError> {
        u32::try_from(bytes.len()).map_err(|_| IntlServiceError::InvalidWire)?;
        let mut reader = Self { bytes, cursor: 0 };
        if reader.word()? != RELATIVE_TIME_WIRE_VERSION || reader.word()? != operation.tag(response)
        {
            return Err(IntlServiceError::InvalidWire);
        }
        Ok(reader)
    }

    fn word(&mut self) -> Result<u64, IntlServiceError> {
        let end = self
            .cursor
            .checked_add(8)
            .ok_or(IntlServiceError::InvalidWire)?;
        let bytes = self
            .bytes
            .get(self.cursor..end)
            .ok_or(IntlServiceError::InvalidWire)?;
        self.cursor = end;
        Ok(u64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| IntlServiceError::InvalidWire)?,
        ))
    }

    fn bytes(&mut self) -> Result<&'a [u8], IntlServiceError> {
        let length = usize::try_from(self.word()?).map_err(|_| IntlServiceError::InvalidWire)?;
        let end = self
            .cursor
            .checked_add(length)
            .ok_or(IntlServiceError::InvalidWire)?;
        let bytes = self
            .bytes
            .get(self.cursor..end)
            .ok_or(IntlServiceError::InvalidWire)?;
        self.cursor = end;
        Ok(bytes)
    }

    fn text(&mut self) -> Result<String, IntlServiceError> {
        core::str::from_utf8(self.bytes()?)
            .map(str::to_owned)
            .map_err(|_| IntlServiceError::InvalidWire)
    }

    fn locale(&mut self) -> Result<CanonicalLocaleId, IntlServiceError> {
        CanonicalLocaleId::from_data(self.text()?.into_boxed_str())
            .map_err(|_| IntlServiceError::InvalidLocale)
    }

    fn locales(&mut self) -> Result<Box<[CanonicalLocaleId]>, IntlServiceError> {
        let count = usize::try_from(self.word()?).map_err(|_| IntlServiceError::InvalidWire)?;
        if count > self.bytes.len().saturating_sub(self.cursor) / 8 {
            return Err(IntlServiceError::InvalidWire);
        }
        let mut output = Vec::new();
        output
            .try_reserve_exact(count)
            .map_err(|_| IntlServiceError::InvalidWire)?;
        for _ in 0..count {
            output.push(self.locale()?);
        }
        Ok(output.into_boxed_slice())
    }

    fn configuration(&mut self) -> Result<ResolvedRelativeTimeFormat, IntlServiceError> {
        let locale = self.locale()?;
        let data_locale = self.locale()?;
        let numbering_system = self.text()?;
        let style = style_from_code(self.word()?)?;
        let numeric = numeric_from_code(self.word()?)?;
        ResolvedRelativeTimeFormat::from_resolved(
            locale,
            data_locale,
            RelativeTimeFormatOptions::new(style, numeric),
            &numbering_system,
        )
    }

    fn input(&mut self) -> Result<ObservedNumericInput, IntlServiceError> {
        match RelativeTimeNumberKind::from_wire_code(self.word()?) {
            Some(RelativeTimeNumberKind::ShortestDecimal) => Ok(
                ObservedNumericInput::NumberShortestDecimal(self.text()?.into_boxed_str()),
            ),
            Some(RelativeTimeNumberKind::NegativeZero) if self.bytes()?.is_empty() => {
                Ok(ObservedNumericInput::NegativeZero)
            }
            _ => Err(IntlServiceError::InvalidWire),
        }
    }

    fn finish(self) -> Result<(), IntlServiceError> {
        (self.cursor == self.bytes.len())
            .then_some(())
            .ok_or(IntlServiceError::InvalidWire)
    }
}

fn matcher_code(value: ServiceLocaleMatcher) -> u64 {
    match value {
        ServiceLocaleMatcher::Lookup => 1,
        ServiceLocaleMatcher::BestFit => 2,
    }
}

fn matcher_from_code(code: u64) -> Result<ServiceLocaleMatcher, IntlServiceError> {
    match code {
        1 => Ok(ServiceLocaleMatcher::Lookup),
        2 => Ok(ServiceLocaleMatcher::BestFit),
        _ => Err(IntlServiceError::InvalidWire),
    }
}

fn style_code(value: RelativeTimeStyle) -> u64 {
    match value {
        RelativeTimeStyle::Long => 1,
        RelativeTimeStyle::Short => 2,
        RelativeTimeStyle::Narrow => 3,
    }
}

fn style_from_code(code: u64) -> Result<RelativeTimeStyle, IntlServiceError> {
    match code {
        1 => Ok(RelativeTimeStyle::Long),
        2 => Ok(RelativeTimeStyle::Short),
        3 => Ok(RelativeTimeStyle::Narrow),
        _ => Err(IntlServiceError::InvalidWire),
    }
}

fn numeric_code(value: RelativeTimeNumeric) -> u64 {
    match value {
        RelativeTimeNumeric::Always => 1,
        RelativeTimeNumeric::Auto => 2,
    }
}

fn numeric_from_code(code: u64) -> Result<RelativeTimeNumeric, IntlServiceError> {
    match code {
        1 => Ok(RelativeTimeNumeric::Always),
        2 => Ok(RelativeTimeNumeric::Auto),
        _ => Err(IntlServiceError::InvalidWire),
    }
}

impl RelativeTimeLocaleRequest {
    pub fn encode(&self) -> Result<Vec<u8>, IntlServiceError> {
        let mut writer = Writer::new(Operation::ResolveLocale, false);
        writer.word(matcher_code(self.matcher()));
        match self.query() {
            RelativeTimeLocaleQuery::Resolve(_) => writer.word(1),
            RelativeTimeLocaleQuery::SupportedLocales => writer.word(2),
        }
        writer.locales(self.requested())?;
        if let RelativeTimeLocaleQuery::Resolve(options) = self.query() {
            writer.text(
                options
                    .numbering_system()
                    .map(NumberingSystemOption::name)
                    .unwrap_or(""),
            )?;
            writer.word(style_code(options.style()));
            writer.word(numeric_code(options.numeric()));
        }
        Ok(writer.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = Reader::new(bytes, Operation::ResolveLocale, false)?;
        let matcher = matcher_from_code(reader.word()?)?;
        let query = reader.word()?;
        let requested = reader.locales()?;
        let query = match query {
            1 => {
                let numbering_system = reader.text()?;
                let numbering_system = (!numbering_system.is_empty())
                    .then(|| NumberingSystemOption::parse(&numbering_system))
                    .transpose()
                    .map_err(|_| IntlServiceError::InvalidOption)?;
                let style = style_from_code(reader.word()?)?;
                let numeric = numeric_from_code(reader.word()?)?;
                RelativeTimeLocaleQuery::Resolve(
                    RelativeTimeFormatOptions::new(style, numeric)
                        .with_numbering_system(numbering_system),
                )
            }
            2 => RelativeTimeLocaleQuery::SupportedLocales,
            _ => return Err(IntlServiceError::InvalidWire),
        };
        reader.finish()?;
        Ok(match query {
            RelativeTimeLocaleQuery::Resolve(options) => Self::resolve(requested, matcher, options),
            RelativeTimeLocaleQuery::SupportedLocales => {
                Self::supported_locales(requested, matcher)
            }
        })
    }
}

impl RelativeTimeLocaleResult {
    pub fn encode(&self) -> Result<Vec<u8>, IntlServiceError> {
        let mut writer = Writer::new(Operation::ResolveLocale, true);
        match self {
            Self::Resolved(configuration) => {
                writer.word(1);
                writer.configuration(configuration)?;
            }
            Self::SupportedLocales(locales) => {
                writer.word(2);
                writer.locales(locales)?;
            }
        }
        Ok(writer.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = Reader::new(bytes, Operation::ResolveLocale, true)?;
        let result = match reader.word()? {
            1 => Self::Resolved(reader.configuration()?),
            2 => Self::SupportedLocales(reader.locales()?),
            _ => return Err(IntlServiceError::InvalidWire),
        };
        reader.finish()?;
        Ok(result)
    }
}

impl RelativeTimeFormatRequest {
    pub fn encode(&self) -> Result<Vec<u8>, IntlServiceError> {
        let mut writer = Writer::new(Operation::Format, false);
        writer.configuration(self.configuration())?;
        writer.word(self.unit().wire_code());
        writer.input(self.input())?;
        Ok(writer.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = Reader::new(bytes, Operation::Format, false)?;
        let configuration = reader.configuration()?;
        let unit = RelativeTimeUnit::from_wire_code(reader.word()?)
            .ok_or(IntlServiceError::InvalidWire)?;
        let input = reader.input()?;
        reader.finish()?;
        Ok(Self::new(configuration, unit, input))
    }
}

impl RelativeTimeParts {
    pub fn encode(&self) -> Result<Vec<u8>, IntlServiceError> {
        let mut writer = Writer::new(Operation::Format, true);
        writer.word(u64::try_from(self.0.len()).map_err(|_| IntlServiceError::InvalidWire)?);
        for part in &self.0 {
            writer.word(part.kind.wire_code());
            writer.text(&part.value)?;
            writer.word(part.unit.wire_code());
        }
        Ok(writer.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = Reader::new(bytes, Operation::Format, true)?;
        let count = usize::try_from(reader.word()?).map_err(|_| IntlServiceError::InvalidWire)?;
        if count > bytes.len().saturating_sub(reader.cursor) / 24 {
            return Err(IntlServiceError::InvalidWire);
        }
        let mut parts = Vec::new();
        parts
            .try_reserve_exact(count)
            .map_err(|_| IntlServiceError::InvalidWire)?;
        for _ in 0..count {
            let kind = RelativeTimePartKind::from_wire_code(reader.word()?)
                .ok_or(IntlServiceError::InvalidWire)?;
            let value = reader.text()?.into_boxed_str();
            let unit = RelativeTimeUnit::from_wire_code(reader.word()?)
                .ok_or(IntlServiceError::InvalidWire)?;
            parts.push(RelativeTimePart { kind, value, unit });
        }
        reader.finish()?;
        Ok(Self(parts.into_boxed_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(source: &str) -> CanonicalLocaleId {
        CanonicalLocaleId::from_data(source).unwrap()
    }

    #[test]
    fn short_supported_locale_request_and_response_round_trip() {
        let request = RelativeTimeLocaleRequest::supported_locales(
            vec![locale("en")].into_boxed_slice(),
            ServiceLocaleMatcher::Lookup,
        );
        let encoded = request.encode().unwrap();
        assert_eq!(
            RelativeTimeLocaleRequest::decode(&encoded).unwrap(),
            request
        );

        let response =
            RelativeTimeLocaleResult::SupportedLocales(vec![locale("en")].into_boxed_slice());
        let encoded = response.encode().unwrap();
        assert_eq!(
            RelativeTimeLocaleResult::decode(&encoded).unwrap(),
            response
        );
    }

    #[test]
    fn malformed_short_locale_counts_are_rejected_without_overread() {
        let request = RelativeTimeLocaleRequest::supported_locales(
            vec![locale("en")].into_boxed_slice(),
            ServiceLocaleMatcher::Lookup,
        );
        let mut encoded_request = request.encode().unwrap();
        encoded_request[32..40].copy_from_slice(&2_u64.to_le_bytes());
        assert_eq!(
            RelativeTimeLocaleRequest::decode(&encoded_request),
            Err(IntlServiceError::InvalidWire)
        );

        let response =
            RelativeTimeLocaleResult::SupportedLocales(vec![locale("en")].into_boxed_slice());
        let mut encoded_response = response.encode().unwrap();
        encoded_response[24..32].copy_from_slice(&2_u64.to_le_bytes());
        assert_eq!(
            RelativeTimeLocaleResult::decode(&encoded_response),
            Err(IntlServiceError::InvalidWire)
        );
    }
}

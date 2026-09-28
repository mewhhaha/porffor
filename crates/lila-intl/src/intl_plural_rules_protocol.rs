//! Closed host messages for the PluralRules locale and category operations.

use crate::NumberNumericKind;
use crate::intl_services::{
    PluralCategoryInput, PluralRulesLocaleQuery, PluralRulesOptions, PluralRulesType,
    ServiceLocaleMatcher,
};
use crate::number_format::numeric::ObservedNumericInput;
use crate::{
    CanonicalLocaleId, IntlHostOp, IntlServiceError, PluralCategory, PluralCategoryRequest,
    PluralRulesLocaleRequest, PluralRulesLocaleResult, ResolvedPluralRules,
};

pub const PLURAL_RULES_WIRE_VERSION: u64 = 2;
pub const PLURAL_RULES_WIRE_HEADER_BYTES: u64 = 16;

#[derive(Clone, Copy)]
enum Operation {
    ResolveLocale,
    SelectCategory,
}

impl Operation {
    fn host(self) -> IntlHostOp {
        match self {
            Self::ResolveLocale => IntlHostOp::ResolvePluralRulesLocale,
            Self::SelectCategory => IntlHostOp::SelectPluralCategory,
        }
    }

    fn request_tag(self) -> u64 {
        u64::from(self.host().code()) * 2
    }
}

struct Writer(Vec<u8>);

impl Writer {
    fn new(operation: Operation, response: bool) -> Result<Self, IntlServiceError> {
        let mut writer = Self(Vec::new());
        writer.word(PLURAL_RULES_WIRE_VERSION)?;
        writer.word(operation.request_tag() + u64::from(response))?;
        Ok(writer)
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), IntlServiceError> {
        let length = self
            .0
            .len()
            .checked_add(bytes.len())
            .ok_or(IntlServiceError::InvalidWire)?;
        u32::try_from(length).map_err(|_| IntlServiceError::InvalidWire)?;
        self.0
            .try_reserve(bytes.len())
            .map_err(|_| IntlServiceError::InvalidWire)?;
        self.0.extend_from_slice(bytes);
        Ok(())
    }

    fn word(&mut self, value: u64) -> Result<(), IntlServiceError> {
        self.append(&value.to_le_bytes())
    }

    fn bytes(&mut self, value: &[u8]) -> Result<(), IntlServiceError> {
        self.word(u64::try_from(value.len()).map_err(|_| IntlServiceError::InvalidWire)?)?;
        self.append(value)
    }

    fn text(&mut self, value: &str) -> Result<(), IntlServiceError> {
        self.bytes(value.as_bytes())
    }

    fn locale(&mut self, locale: &CanonicalLocaleId) -> Result<(), IntlServiceError> {
        self.text(locale.as_str())
    }

    fn locales(&mut self, locales: &[CanonicalLocaleId]) -> Result<(), IntlServiceError> {
        self.word(u64::try_from(locales.len()).map_err(|_| IntlServiceError::InvalidWire)?)?;
        for locale in locales {
            self.locale(locale)?;
        }
        Ok(())
    }

    fn resolved(&mut self, value: &ResolvedPluralRules) -> Result<(), IntlServiceError> {
        self.locale(value.locale())?;
        self.locale(value.data_locale())?;
        self.word(rule_type_code(value.options().rule_type()))?;
        self.word(u64::from(value.category_mask()))?;
        self.word(
            u64::try_from(value.categories().len()).map_err(|_| IntlServiceError::InvalidWire)?,
        )?;
        for word in value.options().wire_words() {
            self.word(word)?;
        }
        Ok(())
    }

    fn input(&mut self, input: &ObservedNumericInput) -> Result<(), IntlServiceError> {
        match input {
            ObservedNumericInput::StringNumericLiteral(units) => {
                self.word(NumberNumericKind::String.wire_code())?;
                let byte_length = units
                    .len()
                    .checked_mul(2)
                    .ok_or(IntlServiceError::InvalidWire)?;
                self.word(u64::try_from(byte_length).map_err(|_| IntlServiceError::InvalidWire)?)?;
                for unit in units {
                    self.append(&unit.to_le_bytes())?;
                }
                Ok(())
            }
            ObservedNumericInput::NumberShortestDecimal(number) => {
                self.word(NumberNumericKind::Number.wire_code())?;
                self.text(number)
            }
            ObservedNumericInput::NegativeZero => {
                self.word(NumberNumericKind::NegativeZero.wire_code())?;
                self.bytes(&[])
            }
            ObservedNumericInput::BigIntDecimal(number) => {
                self.word(NumberNumericKind::BigInt.wire_code())?;
                self.text(number)
            }
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
        if reader.word()? != PLURAL_RULES_WIRE_VERSION
            || reader.word()? != operation.request_tag() + u64::from(response)
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
            bytes.try_into().expect("eight-byte field"),
        ))
    }

    fn bytes(&mut self) -> Result<&'a [u8], IntlServiceError> {
        let length = usize::try_from(self.word()?).map_err(|_| IntlServiceError::InvalidWire)?;
        let end = self
            .cursor
            .checked_add(length)
            .ok_or(IntlServiceError::InvalidWire)?;
        let result = self
            .bytes
            .get(self.cursor..end)
            .ok_or(IntlServiceError::InvalidWire)?;
        self.cursor = end;
        Ok(result)
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
        let mut locales = Vec::new();
        locales
            .try_reserve_exact(count)
            .map_err(|_| IntlServiceError::InvalidWire)?;
        for _ in 0..count {
            locales.push(self.locale()?);
        }
        Ok(locales.into_boxed_slice())
    }

    fn matcher(&mut self) -> Result<ServiceLocaleMatcher, IntlServiceError> {
        match self.word()? {
            1 => Ok(ServiceLocaleMatcher::Lookup),
            2 => Ok(ServiceLocaleMatcher::BestFit),
            _ => Err(IntlServiceError::InvalidWire),
        }
    }

    fn resolved(&mut self) -> Result<ResolvedPluralRules, IntlServiceError> {
        let locale = self.locale()?;
        let data_locale = self.locale()?;
        let rule_type = rule_type_from_code(self.word()?)?;
        let category_mask =
            u8::try_from(self.word()?).map_err(|_| IntlServiceError::InvalidWire)?;
        let category_count =
            usize::try_from(self.word()?).map_err(|_| IntlServiceError::InvalidWire)?;
        let mut words = [0_u64; crate::NUMBER_CONFIGURATION_WORDS];
        for word in &mut words {
            *word = self.word()?;
        }
        let options = PluralRulesOptions::from_wire_words(rule_type, words)?;
        let resolved = ResolvedPluralRules::from_provider(locale, data_locale, options)?;
        if resolved.category_mask() != category_mask
            || resolved.categories().len() != category_count
        {
            return Err(IntlServiceError::InvalidWire);
        }
        Ok(resolved)
    }

    fn input(&mut self) -> Result<ObservedNumericInput, IntlServiceError> {
        match NumberNumericKind::from_wire_code(self.word()?)
            .ok_or(IntlServiceError::InvalidWire)?
        {
            NumberNumericKind::String => {
                let bytes = self.bytes()?;
                if bytes.len() % 2 != 0 {
                    return Err(IntlServiceError::InvalidWire);
                }
                let mut units = Vec::new();
                units
                    .try_reserve_exact(bytes.len() / 2)
                    .map_err(|_| IntlServiceError::InvalidWire)?;
                units.extend(
                    bytes
                        .chunks_exact(2)
                        .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
                );
                Ok(ObservedNumericInput::StringNumericLiteral(
                    units.into_boxed_slice(),
                ))
            }
            NumberNumericKind::Number => Ok(ObservedNumericInput::NumberShortestDecimal(
                self.text()?.into_boxed_str(),
            )),
            NumberNumericKind::NegativeZero if self.bytes()?.is_empty() => {
                Ok(ObservedNumericInput::NegativeZero)
            }
            NumberNumericKind::BigInt => Ok(ObservedNumericInput::BigIntDecimal(
                self.text()?.into_boxed_str(),
            )),
            NumberNumericKind::NegativeZero => Err(IntlServiceError::InvalidWire),
        }
    }

    fn finish(self) -> Result<(), IntlServiceError> {
        (self.cursor == self.bytes.len())
            .then_some(())
            .ok_or(IntlServiceError::InvalidWire)
    }
}

fn rule_type_code(rule_type: PluralRulesType) -> u64 {
    match rule_type {
        PluralRulesType::Cardinal => 1,
        PluralRulesType::Ordinal => 2,
    }
}

fn rule_type_from_code(code: u64) -> Result<PluralRulesType, IntlServiceError> {
    match code {
        1 => Ok(PluralRulesType::Cardinal),
        2 => Ok(PluralRulesType::Ordinal),
        _ => Err(IntlServiceError::InvalidWire),
    }
}

impl PluralRulesLocaleRequest {
    pub fn encode(&self) -> Result<Vec<u8>, IntlServiceError> {
        let mut writer = Writer::new(Operation::ResolveLocale, false)?;
        writer.word(match self.matcher() {
            ServiceLocaleMatcher::Lookup => 1,
            ServiceLocaleMatcher::BestFit => 2,
        })?;
        match self.query() {
            PluralRulesLocaleQuery::Resolve(options) => {
                writer.word(1)?;
                writer.locales(self.requested())?;
                writer.word(rule_type_code(options.rule_type()))?;
                for word in options.wire_words() {
                    writer.word(word)?;
                }
            }
            PluralRulesLocaleQuery::SupportedLocales => {
                writer.word(2)?;
                writer.locales(self.requested())?;
            }
        }
        Ok(writer.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = Reader::new(bytes, Operation::ResolveLocale, false)?;
        let matcher = reader.matcher()?;
        let query = reader.word()?;
        let requested = reader.locales()?;
        let result = match query {
            1 => {
                let rule_type = rule_type_from_code(reader.word()?)?;
                let mut words = [0_u64; crate::NUMBER_CONFIGURATION_WORDS];
                for word in &mut words {
                    *word = reader.word()?;
                }
                Self::resolve(
                    requested,
                    matcher,
                    PluralRulesOptions::from_wire_words(rule_type, words)?,
                )
            }
            2 => Self::supported_locales(requested, matcher),
            _ => return Err(IntlServiceError::InvalidWire),
        };
        reader.finish()?;
        Ok(result)
    }
}

impl PluralRulesLocaleResult {
    pub fn encode(&self) -> Result<Vec<u8>, IntlServiceError> {
        let mut writer = Writer::new(Operation::ResolveLocale, true)?;
        match self {
            Self::Resolved(value) => {
                writer.word(1)?;
                writer.resolved(value)?;
            }
            Self::SupportedLocales(locales) => {
                writer.word(2)?;
                writer.locales(locales)?;
            }
        }
        Ok(writer.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = Reader::new(bytes, Operation::ResolveLocale, true)?;
        let result = match reader.word()? {
            1 => Self::Resolved(reader.resolved()?),
            2 => Self::SupportedLocales(reader.locales()?),
            _ => return Err(IntlServiceError::InvalidWire),
        };
        reader.finish()?;
        Ok(result)
    }
}

impl PluralCategoryRequest {
    pub fn encode(&self) -> Result<Vec<u8>, IntlServiceError> {
        let mut writer = Writer::new(Operation::SelectCategory, false)?;
        writer.resolved(self.configuration())?;
        match self.input() {
            PluralCategoryInput::Single(number) => {
                writer.word(1)?;
                writer.input(number)?;
            }
            PluralCategoryInput::Range { start, end } => {
                writer.word(2)?;
                writer.input(start)?;
                writer.input(end)?;
            }
        }
        Ok(writer.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = Reader::new(bytes, Operation::SelectCategory, false)?;
        let configuration = reader.resolved()?;
        let result = match reader.word()? {
            1 => Self::select(configuration, reader.input()?),
            2 => Self::select_range(configuration, reader.input()?, reader.input()?),
            _ => return Err(IntlServiceError::InvalidWire),
        };
        reader.finish()?;
        Ok(result)
    }
}

impl PluralCategory {
    pub fn encode(&self) -> Result<Vec<u8>, IntlServiceError> {
        let mut writer = Writer::new(Operation::SelectCategory, true)?;
        writer.word(self.wire_code())?;
        Ok(writer.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IntlServiceError> {
        let mut reader = Reader::new(bytes, Operation::SelectCategory, true)?;
        let result = Self::from_wire_code(reader.word()?).ok_or(IntlServiceError::InvalidWire)?;
        reader.finish()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServiceLocaleMatcher;
    use crate::intl_services::PluralRulesPrecision;

    fn locale(source: &str) -> CanonicalLocaleId {
        CanonicalLocaleId::from_data(source).unwrap()
    }

    #[test]
    fn short_supported_locale_request_and_response_round_trip() {
        let request = PluralRulesLocaleRequest::supported_locales(
            vec![locale("en")].into_boxed_slice(),
            ServiceLocaleMatcher::Lookup,
        );
        let encoded = request.encode().unwrap();
        assert_eq!(PluralRulesLocaleRequest::decode(&encoded).unwrap(), request);

        let response =
            PluralRulesLocaleResult::SupportedLocales(vec![locale("en")].into_boxed_slice());
        let encoded = response.encode().unwrap();
        assert_eq!(PluralRulesLocaleResult::decode(&encoded).unwrap(), response);
    }

    fn resolved() -> ResolvedPluralRules {
        let locale = locale("en");
        let options = PluralRulesOptions::new(
            PluralRulesType::Cardinal,
            1,
            PluralRulesPrecision::significant(1, 21).unwrap(),
        )
        .unwrap();
        ResolvedPluralRules::from_provider(locale.clone(), locale, options).unwrap()
    }

    #[test]
    fn exact_string_bigint_and_number_inputs_round_trip() {
        let configuration = resolved();
        let inputs = [
            ObservedNumericInput::StringNumericLiteral(
                "1.0000000000000000001".encode_utf16().collect(),
            ),
            ObservedNumericInput::StringNumericLiteral(
                vec![u16::from(b'1'), 0xd800].into_boxed_slice(),
            ),
            ObservedNumericInput::NumberShortestDecimal("1.5".into()),
            ObservedNumericInput::NegativeZero,
            ObservedNumericInput::BigIntDecimal("100000000000000000001".into()),
        ];
        for input in inputs {
            let request = PluralCategoryRequest::select(configuration.clone(), input);
            let encoded = request.encode().unwrap();
            assert_eq!(PluralCategoryRequest::decode(&encoded).unwrap(), request);
        }

        let request = PluralCategoryRequest::select_range(
            configuration,
            ObservedNumericInput::StringNumericLiteral("1".encode_utf16().collect()),
            ObservedNumericInput::BigIntDecimal("2".into()),
        );
        let encoded = request.encode().unwrap();
        assert_eq!(PluralCategoryRequest::decode(&encoded).unwrap(), request);
    }

    #[test]
    fn odd_utf16_numeric_string_wire_payload_is_rejected() {
        let request = PluralCategoryRequest::select(
            resolved(),
            ObservedNumericInput::StringNumericLiteral(vec![u16::from(b'1')].into_boxed_slice()),
        );
        let mut encoded = request.encode().unwrap();
        let length_offset = encoded.len() - 10;
        encoded[length_offset..length_offset + 8].copy_from_slice(&1_u64.to_le_bytes());
        encoded.pop();
        assert_eq!(
            PluralCategoryRequest::decode(&encoded),
            Err(IntlServiceError::InvalidWire)
        );
    }

    #[test]
    fn malformed_short_locale_counts_are_rejected() {
        let request = PluralRulesLocaleRequest::supported_locales(
            vec![locale("en")].into_boxed_slice(),
            ServiceLocaleMatcher::Lookup,
        );
        let mut encoded_request = request.encode().unwrap();
        encoded_request[32..40].copy_from_slice(&2_u64.to_le_bytes());
        assert_eq!(
            PluralRulesLocaleRequest::decode(&encoded_request),
            Err(IntlServiceError::InvalidWire)
        );

        let response =
            PluralRulesLocaleResult::SupportedLocales(vec![locale("en")].into_boxed_slice());
        let mut encoded_response = response.encode().unwrap();
        encoded_response[24..32].copy_from_slice(&2_u64.to_le_bytes());
        assert_eq!(
            PluralRulesLocaleResult::decode(&encoded_response),
            Err(IntlServiceError::InvalidWire)
        );
    }
}

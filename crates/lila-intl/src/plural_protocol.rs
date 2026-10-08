//! Lossless primitive PluralRules messages; numeric fields share NumberFormat's codec.
use crate::number_format::options::*;
use crate::number_format::NumberProfiles;
use crate::number_protocol::{
    decode_rounding_words, rounding_wire_words, NumberWireDirection, NumberWireError,
    NumberWireReader, NumberWireWriter,
};
use crate::plural_rules::*;
use crate::IntlHostOp;
use core::fmt;
use std::sync::Arc;

pub const PLURAL_WIRE_VERSION: u64 = 1;
pub const PLURAL_WIRE_HEADER_BYTES: u64 = 16;
pub const PLURAL_CONFIGURATION_WORDS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralConfigurationWord {
    Type,
    Notation,
    CompactDisplay,
    MinimumInteger,
    Precision,
    MinimumFraction,
    MaximumFraction,
    MinimumSignificant,
    MaximumSignificant,
    RoundingIncrement,
    RoundingMode,
    TrailingZero,
}
impl PluralConfigurationWord {
    pub const ALL: [Self; PLURAL_CONFIGURATION_WORDS] = [
        Self::Type,
        Self::Notation,
        Self::CompactDisplay,
        Self::MinimumInteger,
        Self::Precision,
        Self::MinimumFraction,
        Self::MaximumFraction,
        Self::MinimumSignificant,
        Self::MaximumSignificant,
        Self::RoundingIncrement,
        Self::RoundingMode,
        Self::TrailingZero,
    ];
    pub const fn index(self) -> usize {
        match self {
            Self::Type => 0,
            Self::Notation => 1,
            Self::CompactDisplay => 2,
            Self::MinimumInteger => 3,
            Self::Precision => 4,
            Self::MinimumFraction => 5,
            Self::MaximumFraction => 6,
            Self::MinimumSignificant => 7,
            Self::MaximumSignificant => 8,
            Self::RoundingIncrement => 9,
            Self::RoundingMode => 10,
            Self::TrailingZero => 11,
        }
    }
    pub const fn offset(self) -> u64 {
        self.index() as u64 * 8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluralWireError {
    Primitive(NumberWireError),
    Operation(PluralRulesOperationError),
}
impl fmt::Display for PluralWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(e) => e.fmt(f),
            Self::Operation(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for PluralWireError {}
impl From<NumberWireError> for PluralWireError {
    fn from(e: NumberWireError) -> Self {
        Self::Primitive(e)
    }
}
impl From<PluralRulesOperationError> for PluralWireError {
    fn from(e: PluralRulesOperationError) -> Self {
        Self::Operation(e)
    }
}

use NumberWireDirection::{Request, Response};
use PluralConfigurationWord as W;

impl CheckedPluralConfiguration {
    pub fn wire_words(&self) -> [u64; PLURAL_CONFIGURATION_WORDS] {
        let mut words = [0; PLURAL_CONFIGURATION_WORDS];
        words[W::Type.index()] = self.plural_type().wire_code();
        words[W::Notation.index()] = match self.notation() {
            Notation::Standard => NotationOption::Standard.wire_code(),
            Notation::Scientific => NotationOption::Scientific.wire_code(),
            Notation::Engineering => NotationOption::Engineering.wire_code(),
            Notation::Compact(display) => {
                words[W::CompactDisplay.index()] = display.wire_code();
                NotationOption::Compact.wire_code()
            }
        };
        let rounding = rounding_wire_words(self.rounding());
        for (index, value) in rounding.into_iter().enumerate() {
            words[index + 3] = value;
        }
        words
    }
}
fn write_configuration(
    writer: &mut NumberWireWriter,
    configuration: &CheckedPluralConfiguration,
) -> Result<(), PluralWireError> {
    writer.text(configuration.locale().resolved().as_str())?;
    writer.text(configuration.locale().data().as_str())?;
    for word in configuration.wire_words() {
        writer.word(word)?;
    }
    Ok(())
}
fn read_locale(
    reader: &mut NumberWireReader<'_>,
    profiles: &Arc<NumberProfiles>,
) -> Result<ResolvedPluralLocale, PluralWireError> {
    let resolved = reader.locale()?;
    let data = reader.locale()?;
    Ok(ResolvedPluralLocale::from_resolved(
        resolved, data, profiles,
    )?)
}
fn read_configuration(
    reader: &mut NumberWireReader<'_>,
    profiles: &Arc<NumberProfiles>,
) -> Result<CheckedPluralConfiguration, PluralWireError> {
    let locale = read_locale(reader, profiles)?;
    let mut words = [0; PLURAL_CONFIGURATION_WORDS];
    for field in W::ALL {
        words[field.index()] = reader.word()?;
    }
    let kind = PluralType::from_wire_code(words[W::Type.index()])
        .ok_or(NumberWireError::Malformed("plural type"))?;
    let notation_code = NotationOption::from_wire_code(words[W::Notation.index()])
        .ok_or(NumberWireError::Malformed("plural notation"))?;
    let display = words[W::CompactDisplay.index()];
    let notation = match notation_code {
        NotationOption::Compact => Notation::Compact(
            CompactDisplay::from_wire_code(display)
                .ok_or(NumberWireError::Malformed("compact display"))?,
        ),
        NotationOption::Standard | NotationOption::Scientific | NotationOption::Engineering => {
            if display != 0 {
                return Err(NumberWireError::Malformed("inactive compact display").into());
            }
            match notation_code {
                NotationOption::Standard => Notation::Standard,
                NotationOption::Scientific => Notation::Scientific,
                NotationOption::Engineering => Notation::Engineering,
                NotationOption::Compact => unreachable!("handled above"),
            }
        }
    };
    let rounding = decode_rounding_words(words[3..].try_into().expect("nine rounding fields"))?;
    Ok(CheckedPluralConfiguration::new(
        locale, kind, notation, rounding,
    ))
}
macro_rules! locale_request {
    ($ty:ty, $op:ident) => {
        impl $ty {
            pub fn encode(&self) -> Result<Vec<u8>, PluralWireError> {
                let mut writer = NumberWireWriter::new(IntlHostOp::$op, Request)?;
                writer.locales(&self.requested)?;
                writer.word(self.matcher.wire_code())?;
                Ok(writer.finish())
            }
            pub fn decode(bytes: &[u8]) -> Result<Self, PluralWireError> {
                let mut reader = NumberWireReader::new(bytes, IntlHostOp::$op, Request)?;
                let requested = reader.locales()?;
                let matcher = reader.domain(LocaleMatcher::from_wire_code)?;
                reader.finish()?;
                Ok(Self { requested, matcher })
            }
        }
    };
}
locale_request!(PluralLocaleRequest, ResolvePluralLocale);
locale_request!(PluralSupportedLocalesRequest, SupportedPluralLocales);
impl ResolvedPluralLocale {
    pub fn encode(&self) -> Result<Vec<u8>, PluralWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::ResolvePluralLocale, Response)?;
        writer.text(self.resolved().as_str())?;
        writer.text(self.data().as_str())?;
        writer.word(self.categories(PluralType::Cardinal).wire_mask())?;
        writer.word(self.categories(PluralType::Ordinal).wire_mask())?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8], profiles: &Arc<NumberProfiles>) -> Result<Self, PluralWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::ResolvePluralLocale, Response)?;
        let locale = read_locale(&mut reader, profiles)?;
        for kind in PluralType::ALL {
            let categories = PluralCategorySet::from_wire_mask(reader.word()?)
                .ok_or(NumberWireError::Malformed("plural category mask"))?;
            if categories != locale.categories(kind) {
                return Err(NumberWireError::Malformed("plural category association").into());
            }
        }
        reader.finish()?;
        Ok(locale)
    }
}
impl PluralSupportedLocalesResult {
    pub fn encode(&self) -> Result<Vec<u8>, PluralWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::SupportedPluralLocales, Response)?;
        writer.locales(&self.locales)?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, PluralWireError> {
        let mut reader =
            NumberWireReader::new(bytes, IntlHostOp::SupportedPluralLocales, Response)?;
        let locales = reader.locales()?;
        reader.finish()?;
        Ok(Self { locales })
    }
}
impl SelectPluralRequest {
    pub fn encode(&self) -> Result<Vec<u8>, PluralWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::SelectPlural, Request)?;
        write_configuration(&mut writer, self.configuration())?;
        writer.input(self.input())?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8], profiles: &Arc<NumberProfiles>) -> Result<Self, PluralWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::SelectPlural, Request)?;
        let configuration = read_configuration(&mut reader, profiles)?;
        let input = reader.input()?;
        reader.finish()?;
        Ok(Self::new(configuration, input))
    }
}
impl SelectPluralRangeRequest {
    pub fn encode(&self) -> Result<Vec<u8>, PluralWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::SelectPluralRange, Request)?;
        write_configuration(&mut writer, self.configuration())?;
        writer.input(self.start())?;
        writer.input(self.end())?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8], profiles: &Arc<NumberProfiles>) -> Result<Self, PluralWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::SelectPluralRange, Request)?;
        let configuration = read_configuration(&mut reader, profiles)?;
        let start = reader.input()?;
        let end = reader.input()?;
        reader.finish()?;
        Ok(Self::new(configuration, start, end))
    }
}
macro_rules! category_response {
    ($encode:ident, $decode:ident, $op:ident) => {
        impl PluralCategory {
            pub fn $encode(self) -> Result<Vec<u8>, PluralWireError> {
                let mut writer = NumberWireWriter::new(IntlHostOp::$op, Response)?;
                writer.word(self.wire_code())?;
                Ok(writer.finish())
            }
            pub fn $decode(bytes: &[u8]) -> Result<Self, PluralWireError> {
                let mut reader = NumberWireReader::new(bytes, IntlHostOp::$op, Response)?;
                let category = reader.domain(Self::from_wire_code)?;
                reader.finish()?;
                Ok(category)
            }
        }
    };
}
category_response!(encode_scalar, decode_scalar, SelectPlural);
category_response!(encode_range, decode_range, SelectPluralRange);

const _: () = {
    let mut index = 0;
    while index < PluralConfigurationWord::ALL.len() {
        assert!(PluralConfigurationWord::ALL[index].index() == index);
        index += 1;
    }
    assert!(PLURAL_WIRE_VERSION == crate::NUMBER_WIRE_VERSION);
    assert!(PLURAL_WIRE_HEADER_BYTES == crate::NUMBER_WIRE_HEADER_BYTES);
};
#[cfg(test)]
mod tests;

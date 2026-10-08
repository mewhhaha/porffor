//! Header and private-record field vocabulary for the Collator wire.

pub const COLLATOR_WIRE_VERSION: u64 = 1;
pub const COLLATOR_HEADER_BYTES: u64 = 16;
pub const COLLATOR_CONFIGURATION_WORDS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollatorHeaderWord {
    Version,
    OperationDirection,
}

impl CollatorHeaderWord {
    pub const ALL: [Self; 2] = [Self::Version, Self::OperationDirection];
    pub const fn index(self) -> usize {
        match self {
            Self::Version => 0,
            Self::OperationDirection => 1,
        }
    }
    pub const fn offset(self) -> u64 {
        self.index() as u64 * 8
    }
}

/// Logical record words. A UnicodeType counted string follows CollationKind in
/// the compare wire; offsets here address the six-word retained record, not a
/// fictional fixed-width request past that optional string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollatorConfigurationWord {
    Usage,
    CollationKind,
    Numeric,
    CaseFirst,
    Sensitivity,
    IgnorePunctuation,
}

impl CollatorConfigurationWord {
    pub const ALL: [Self; COLLATOR_CONFIGURATION_WORDS] = [
        Self::Usage,
        Self::CollationKind,
        Self::Numeric,
        Self::CaseFirst,
        Self::Sensitivity,
        Self::IgnorePunctuation,
    ];
    pub const fn index(self) -> usize {
        match self {
            Self::Usage => 0,
            Self::CollationKind => 1,
            Self::Numeric => 2,
            Self::CaseFirst => 3,
            Self::Sensitivity => 4,
            Self::IgnorePunctuation => 5,
        }
    }
    pub const fn offset(self) -> u64 {
        self.index() as u64 * 8
    }
}

use crate::collator::*;
use crate::number_format::options::LocaleMatcher;
use crate::number_protocol::{
    NumberWireDirection, NumberWireError, NumberWireReader, NumberWireWriter, NUMBER_WIRE_VERSION,
};
use crate::IntlHostOp;
use core::fmt;
const _: () = assert!(COLLATOR_WIRE_VERSION == NUMBER_WIRE_VERSION);
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollatorWireError {
    Primitive(NumberWireError),
    Operation(CollatorOperationError),
}
impl fmt::Display for CollatorWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(error) => error.fmt(f),
            Self::Operation(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for CollatorWireError {}
impl From<NumberWireError> for CollatorWireError {
    fn from(error: NumberWireError) -> Self {
        Self::Primitive(error)
    }
}
impl From<CollatorOperationError> for CollatorWireError {
    fn from(error: CollatorOperationError) -> Self {
        Self::Operation(error)
    }
}
use NumberWireDirection::{Request, Response};
fn boolean(value: u64) -> Option<bool> {
    match value {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}
fn option_boolean(reader: &mut NumberWireReader<'_>) -> Result<Option<bool>, CollatorWireError> {
    Ok(if reader.domain(boolean)? {
        Some(reader.domain(boolean)?)
    } else {
        None
    })
}
fn write_optional_boolean(
    writer: &mut NumberWireWriter,
    value: Option<bool>,
) -> Result<(), CollatorWireError> {
    writer.word(u64::from(value.is_some()))?;
    if let Some(value) = value {
        writer.word(u64::from(value))?;
    }
    Ok(())
}
fn write_collation(
    writer: &mut NumberWireWriter,
    locale: &ResolvedCollatorLocale,
) -> Result<(), CollatorWireError> {
    writer.word(locale.collation_kind().wire_code())?;
    if let Some(value) = locale.collation() {
        writer.text(value)?;
    }
    Ok(())
}
fn read_collation(
    reader: &mut NumberWireReader<'_>,
) -> Result<Option<Box<str>>, CollatorWireError> {
    Ok(
        match reader.domain(CollatorCollationKind::from_wire_code)? {
            CollatorCollationKind::Default => None,
            CollatorCollationKind::UnicodeType => Some(reader.owned_text()?),
        },
    )
}
impl CollatorLocaleRequest {
    pub fn encode(&self) -> Result<Vec<u8>, CollatorWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::ResolveCollatorLocale, Request)?;
        writer.locales(&self.requested)?;
        writer.word(self.matcher.wire_code())?;
        writer.word(self.usage.wire_code())?;
        writer.word(u64::from(self.collation.is_some()))?;
        if let Some(value) = &self.collation {
            writer.text(value.as_str())?;
        }
        write_optional_boolean(&mut writer, self.numeric)?;
        writer.word(u64::from(self.case_first.is_some()))?;
        if let Some(value) = self.case_first {
            writer.word(value.wire_code())?;
        }
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, CollatorWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::ResolveCollatorLocale, Request)?;
        let requested = reader.locales()?;
        let matcher = reader.domain(LocaleMatcher::from_wire_code)?;
        let usage = reader.domain(CollatorUsage::from_wire_code)?;
        let collation = if reader.domain(boolean)? {
            Some(CollatorCollationOption::parse(reader.owned_text()?)?)
        } else {
            None
        };
        let numeric = option_boolean(&mut reader)?;
        let case_first = if reader.domain(boolean)? {
            Some(reader.domain(CollatorCaseFirst::from_wire_code)?)
        } else {
            None
        };
        reader.finish()?;
        Ok(Self {
            requested,
            matcher,
            usage,
            collation,
            numeric,
            case_first,
        })
    }
}
impl CollatorSupportedLocalesRequest {
    pub fn encode(&self) -> Result<Vec<u8>, CollatorWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::SupportedCollatorLocales, Request)?;
        writer.locales(&self.requested)?;
        writer.word(self.matcher.wire_code())?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, CollatorWireError> {
        let mut reader =
            NumberWireReader::new(bytes, IntlHostOp::SupportedCollatorLocales, Request)?;
        let requested = reader.locales()?;
        let matcher = reader.domain(LocaleMatcher::from_wire_code)?;
        reader.finish()?;
        Ok(Self { requested, matcher })
    }
}
impl CollatorSupportedLocalesResult {
    pub fn encode(&self) -> Result<Vec<u8>, CollatorWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::SupportedCollatorLocales, Response)?;
        writer.locales(&self.locales)?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, CollatorWireError> {
        let mut reader =
            NumberWireReader::new(bytes, IntlHostOp::SupportedCollatorLocales, Response)?;
        let locales = reader.locales()?;
        reader.finish()?;
        Ok(Self { locales })
    }
}
impl ResolvedCollatorLocale {
    pub fn encode(&self) -> Result<Vec<u8>, CollatorWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::ResolveCollatorLocale, Response)?;
        writer.text(self.resolved().as_str())?;
        write_collation(&mut writer, self)?;
        writer.word(u64::from(self.numeric()))?;
        writer.word(self.case_first().wire_code())?;
        writer.word(self.default_sensitivity().wire_code())?;
        writer.word(u64::from(self.default_ignore_punctuation()))?;
        Ok(writer.finish())
    }
    /// Usage comes only from the actual matched resolve request, not a response field.
    pub fn decode(
        bytes: &[u8],
        profiles: &CollatorProfiles,
        request: &CollatorLocaleRequest,
    ) -> Result<Self, CollatorWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::ResolveCollatorLocale, Response)?;
        let resolved = reader.locale()?;
        let co = read_collation(&mut reader)?;
        let numeric = reader.domain(boolean)?;
        let case_first = reader.domain(CollatorCaseFirst::from_wire_code)?;
        let sensitivity = reader.domain(CollatorSensitivity::from_wire_code)?;
        let ignore = reader.domain(boolean)?;
        reader.finish()?;
        let result = profiles.admit(resolved, request.usage, co.as_deref(), numeric, case_first)?;
        if sensitivity != result.default_sensitivity()
            || ignore != result.default_ignore_punctuation()
        {
            return Err(CollatorOperationError::InvalidConfiguration.into());
        }
        // Responses must belong to this actual option/extension resolution, not
        // merely another valid configuration in the same immutable catalogue.
        let expected = profiles.resolve(request.clone())?;
        if result.resolved() != expected.resolved()
            || result.collation() != expected.collation()
            || result.numeric() != expected.numeric()
            || result.case_first() != expected.case_first()
        {
            return Err(CollatorOperationError::InvalidConfiguration.into());
        }
        Ok(result)
    }
}
impl CheckedCollatorConfiguration {
    pub fn wire_words(&self) -> [u64; COLLATOR_CONFIGURATION_WORDS] {
        [
            self.locale().usage().wire_code(),
            self.locale().collation_kind().wire_code(),
            u64::from(self.locale().numeric()),
            self.locale().case_first().wire_code(),
            self.sensitivity().wire_code(),
            u64::from(self.ignore_punctuation()),
        ]
    }
}
impl CompareCollatorRequest {
    pub fn encode(&self) -> Result<Vec<u8>, CollatorWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::CompareCollator, Request)?;
        let config = self.configuration();
        let locale = config.locale();
        writer.text(locale.resolved().as_str())?;
        for (field, word) in CollatorConfigurationWord::ALL
            .into_iter()
            .zip(config.wire_words())
        {
            writer.word(word)?;
            match field {
                CollatorConfigurationWord::CollationKind => {
                    if let Some(value) = locale.collation() {
                        writer.text(value)?;
                    }
                }
                CollatorConfigurationWord::Usage
                | CollatorConfigurationWord::Numeric
                | CollatorConfigurationWord::CaseFirst
                | CollatorConfigurationWord::Sensitivity
                | CollatorConfigurationWord::IgnorePunctuation => {}
            }
        }
        writer.utf16_units(self.left())?;
        writer.utf16_units(self.right())?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8], profiles: &CollatorProfiles) -> Result<Self, CollatorWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::CompareCollator, Request)?;
        let locale = reader.locale()?;
        let usage = reader.domain(CollatorUsage::from_wire_code)?;
        let co = read_collation(&mut reader)?;
        let numeric = reader.domain(boolean)?;
        let case_first = reader.domain(CollatorCaseFirst::from_wire_code)?;
        let sensitivity = reader.domain(CollatorSensitivity::from_wire_code)?;
        let ignore = reader.domain(boolean)?;
        let left = reader.utf16_units()?;
        let right = reader.utf16_units()?;
        reader.finish()?;
        let locale = profiles.admit(locale, usage, co.as_deref(), numeric, case_first)?;
        Ok(Self::new(
            CheckedCollatorConfiguration::new(locale, sensitivity, ignore),
            left,
            right,
        )?)
    }
}
impl CollatorOrdering {
    pub fn encode(self) -> Result<Vec<u8>, CollatorWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::CompareCollator, Response)?;
        writer.word(self.wire_code())?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, CollatorWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::CompareCollator, Response)?;
        let value = reader.domain(Self::from_wire_code)?;
        reader.finish()?;
        Ok(value)
    }
}

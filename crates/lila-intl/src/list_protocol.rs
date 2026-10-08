//! Exact UTF16 list inputs and checked indexed partition responses.
use crate::list_format::*;
use crate::number_format::options::LocaleMatcher;
use crate::number_protocol::{
    NumberWireDirection, NumberWireError, NumberWireReader, NumberWireWriter, NUMBER_WIRE_VERSION,
};
use crate::IntlHostOp;
use core::fmt;
pub const LIST_WIRE_VERSION: u64 = 1;
pub const LIST_HEADER_BYTES: u64 = 16;
pub const LIST_FORMAT_CONFIGURATION_WORDS: usize = 2;
const _: () = assert!(LIST_WIRE_VERSION == NUMBER_WIRE_VERSION);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListHeaderWord {
    Version,
    OperationDirection,
}
impl ListHeaderWord {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListFormatConfigurationWord {
    Type,
    Style,
}
impl ListFormatConfigurationWord {
    pub const ALL: [Self; LIST_FORMAT_CONFIGURATION_WORDS] = [Self::Type, Self::Style];
    pub const fn index(self) -> usize {
        match self {
            Self::Type => 0,
            Self::Style => 1,
        }
    }
    pub const fn offset(self) -> u64 {
        self.index() as u64 * 8
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListWireError {
    Primitive(NumberWireError),
    Operation(ListFormatOperationError),
}
impl fmt::Display for ListWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(e) => e.fmt(f),
            Self::Operation(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for ListWireError {}
impl From<NumberWireError> for ListWireError {
    fn from(e: NumberWireError) -> Self {
        Self::Primitive(e)
    }
}
impl From<ListFormatOperationError> for ListWireError {
    fn from(e: ListFormatOperationError) -> Self {
        Self::Operation(e)
    }
}
use NumberWireDirection::{Request, Response};
macro_rules! locale_request {
    ($ty:ty, $op:ident) => {
        impl $ty {
            pub fn encode(&self) -> Result<Vec<u8>, ListWireError> {
                let mut w = NumberWireWriter::new(IntlHostOp::$op, Request)?;
                w.locales(&self.requested)?;
                w.word(self.matcher.wire_code())?;
                Ok(w.finish())
            }
            pub fn decode(bytes: &[u8]) -> Result<Self, ListWireError> {
                let mut r = NumberWireReader::new(bytes, IntlHostOp::$op, Request)?;
                let requested = r.locales()?;
                let matcher = r.domain(LocaleMatcher::from_wire_code)?;
                r.finish()?;
                Ok(Self { requested, matcher })
            }
        }
    };
}
locale_request!(ListLocaleRequest, ResolveListLocale);
locale_request!(ListSupportedLocalesRequest, SupportedListLocales);
impl ResolvedListLocale {
    pub fn encode(&self) -> Result<Vec<u8>, ListWireError> {
        let mut w = NumberWireWriter::new(IntlHostOp::ResolveListLocale, Response)?;
        w.text(self.resolved().as_str())?;
        Ok(w.finish())
    }
    pub fn decode(bytes: &[u8], profiles: &ListProfiles) -> Result<Self, ListWireError> {
        let mut r = NumberWireReader::new(bytes, IntlHostOp::ResolveListLocale, Response)?;
        let locale = profiles.admit(r.locale()?)?;
        r.finish()?;
        Ok(locale)
    }
}
impl ListSupportedLocalesResult {
    pub fn encode(&self) -> Result<Vec<u8>, ListWireError> {
        let mut w = NumberWireWriter::new(IntlHostOp::SupportedListLocales, Response)?;
        w.locales(&self.locales)?;
        Ok(w.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, ListWireError> {
        let mut r = NumberWireReader::new(bytes, IntlHostOp::SupportedListLocales, Response)?;
        let locales = r.locales()?;
        r.finish()?;
        Ok(Self { locales })
    }
}
impl CheckedListConfiguration {
    pub fn wire_words(&self) -> [u64; LIST_FORMAT_CONFIGURATION_WORDS] {
        [self.list_type().wire_code(), self.style().wire_code()]
    }
}
impl FormatListPartsRequest {
    pub fn encode(&self) -> Result<Vec<u8>, ListWireError> {
        let mut w = NumberWireWriter::new(IntlHostOp::FormatListParts, Request)?;
        w.text(self.configuration().locale().resolved().as_str())?;
        for word in self.configuration().wire_words() {
            w.word(word)?;
        }
        w.word(self.elements().len() as u64)?;
        for element in self.elements() {
            w.utf16_units(element)?;
        }
        Ok(w.finish())
    }
    pub fn decode(bytes: &[u8], profiles: &ListProfiles) -> Result<Self, ListWireError> {
        let mut r = NumberWireReader::new(bytes, IntlHostOp::FormatListParts, Request)?;
        let locale = profiles.admit(r.locale()?)?;
        let kind = r.domain(ListType::from_wire_code)?;
        let style = r.domain(ListStyle::from_wire_code)?;
        let count = r.count(8)?;
        let mut elements = Vec::new();
        elements
            .try_reserve_exact(count)
            .map_err(|_| NumberWireError::Resource("element list allocation"))?;
        for _ in 0..count {
            elements.push(r.utf16_units()?);
        }
        r.finish()?;
        Ok(Self::new(
            CheckedListConfiguration::new(locale, kind, style),
            elements.into_boxed_slice(),
        )?)
    }
}
impl ListParts {
    pub fn encode(&self) -> Result<Vec<u8>, ListWireError> {
        let mut w = NumberWireWriter::new(IntlHostOp::FormatListParts, Response)?;
        w.word(self.parts().len() as u64)?;
        for part in self.parts() {
            w.word(part.kind().wire_code())?;
            match part {
                ListPart::Literal(units) => w.utf16_units(units)?,
                ListPart::Element(index) => w.word(u64::from(*index))?,
            }
        }
        Ok(w.finish())
    }
    pub fn decode(bytes: &[u8], input_count: u32) -> Result<Self, ListWireError> {
        let mut r = NumberWireReader::new(bytes, IntlHostOp::FormatListParts, Response)?;
        let count = r.count(16)?;
        let mut parts = Vec::new();
        parts
            .try_reserve_exact(count)
            .map_err(|_| NumberWireError::Resource("parts allocation"))?;
        for _ in 0..count {
            parts.push(match r.domain(ListPartKind::from_wire_code)? {
                ListPartKind::Literal => ListPart::Literal(r.utf16_units()?),
                ListPartKind::Element => ListPart::Element(
                    u32::try_from(r.word()?)
                        .map_err(|_| NumberWireError::Malformed("element index exceeds Wasm32"))?,
                ),
            });
        }
        r.finish()?;
        Ok(Self::checked(parts, input_count)?)
    }
}

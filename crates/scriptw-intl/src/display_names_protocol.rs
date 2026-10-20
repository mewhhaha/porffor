//! Lossless primitive DisplayNames frames. Tags 27..29 are reserved by the
//! shared-operation owner; later global rows must map to these exact tags.
use crate::display_names::*;
use crate::number_format::options::LocaleMatcher;
use crate::CanonicalLocaleId;
use core::fmt;

pub const DISPLAY_NAMES_WIRE_VERSION: u64 = 1;
pub const DISPLAY_NAMES_HEADER_BYTES: u64 = 16;
pub const DISPLAY_NAMES_CONFIGURATION_WORDS: usize = 4;
const _: () = assert!(DISPLAY_NAMES_WIRE_VERSION == crate::number_protocol::NUMBER_WIRE_VERSION);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayNamesConfigurationWord {
    Type,
    Style,
    Fallback,
    LanguageDisplay,
}
impl DisplayNamesConfigurationWord {
    pub const ALL: [Self; DISPLAY_NAMES_CONFIGURATION_WORDS] = [
        Self::Type,
        Self::Style,
        Self::Fallback,
        Self::LanguageDisplay,
    ];
    pub const fn index(self) -> usize {
        match self {
            Self::Type => 0,
            Self::Style => 1,
            Self::Fallback => 2,
            Self::LanguageDisplay => 3,
        }
    }
    pub const fn offset(self) -> u64 {
        self.index() as u64 * 8
    }
}
impl CheckedDisplayNamesConfiguration {
    pub fn wire_words(&self) -> [u64; DISPLAY_NAMES_CONFIGURATION_WORDS] {
        let mut words = [0; DISPLAY_NAMES_CONFIGURATION_WORDS];
        for word in DisplayNamesConfigurationWord::ALL {
            words[word.index()] = match word {
                DisplayNamesConfigurationWord::Type => self.selection().kind().wire_code(),
                DisplayNamesConfigurationWord::Style => self.style().wire_code(),
                DisplayNamesConfigurationWord::Fallback => self.fallback().wire_code(),
                DisplayNamesConfigurationWord::LanguageDisplay => self
                    .selection()
                    .language_display()
                    .map_or(0, DisplayNamesLanguageDisplay::wire_code),
            };
        }
        words
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayNamesWireOperation {
    ResolveLocale,
    SupportedLocales,
    DisplayName,
}
impl DisplayNamesWireOperation {
    pub const ALL: &'static [Self] = &[
        Self::ResolveLocale,
        Self::SupportedLocales,
        Self::DisplayName,
    ];
    pub const fn code(self) -> u16 {
        match self {
            Self::ResolveLocale => 27,
            Self::SupportedLocales => 28,
            Self::DisplayName => 29,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayNamesWireError {
    Malformed(&'static str),
    Native(DisplayNamesError),
    Resource(&'static str),
}
impl From<DisplayNamesError> for DisplayNamesWireError {
    fn from(value: DisplayNamesError) -> Self {
        Self::Native(value)
    }
}
impl fmt::Display for DisplayNamesWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(reason) => write!(f, "invalid DisplayNames frame: {reason}"),
            Self::Native(error) => error.fmt(f),
            Self::Resource(reason) => write!(f, "DisplayNames wire resource: {reason}"),
        }
    }
}
impl std::error::Error for DisplayNamesWireError {}

struct Writer(Vec<u8>);
impl Writer {
    fn new(op: DisplayNamesWireOperation, response: bool) -> Result<Self, DisplayNamesWireError> {
        let mut result = Self(Vec::new());
        result.word(DISPLAY_NAMES_WIRE_VERSION)?;
        result.word(u64::from(op.code()) * 2 + u64::from(response))?;
        Ok(result)
    }
    fn append(&mut self, data: &[u8]) -> Result<(), DisplayNamesWireError> {
        let size = self
            .0
            .len()
            .checked_add(data.len())
            .ok_or(DisplayNamesWireError::Resource("extent"))?;
        u32::try_from(size).map_err(|_| DisplayNamesWireError::Resource("frame exceeds Wasm32"))?;
        self.0
            .try_reserve(data.len())
            .map_err(|_| DisplayNamesWireError::Resource("allocation"))?;
        self.0.extend_from_slice(data);
        Ok(())
    }
    fn word(&mut self, word: u64) -> Result<(), DisplayNamesWireError> {
        self.append(&word.to_le_bytes())
    }
    fn text(&mut self, text: &str) -> Result<(), DisplayNamesWireError> {
        self.word(text.len() as u64)?;
        self.append(text.as_bytes())
    }
    fn locales(&mut self, locales: &[CanonicalLocaleId]) -> Result<(), DisplayNamesWireError> {
        self.word(locales.len() as u64)?;
        for locale in locales {
            self.text(locale.as_str())?;
        }
        Ok(())
    }
    fn configuration(
        &mut self,
        configuration: &CheckedDisplayNamesConfiguration,
    ) -> Result<(), DisplayNamesWireError> {
        self.text(configuration.locale().resolved().as_str())?;
        for word in configuration.wire_words() {
            self.word(word)?;
        }
        Ok(())
    }
    fn utf16(&mut self, units: &[u16]) -> Result<(), DisplayNamesWireError> {
        let bytes = units
            .len()
            .checked_mul(2)
            .ok_or(DisplayNamesWireError::Resource("UTF16 extent"))?;
        self.word(bytes as u64)?;
        for unit in units {
            self.append(&unit.to_le_bytes())?;
        }
        Ok(())
    }
}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn new(
        data: &'a [u8],
        op: DisplayNamesWireOperation,
        response: bool,
    ) -> Result<Self, DisplayNamesWireError> {
        u32::try_from(data.len())
            .map_err(|_| DisplayNamesWireError::Resource("frame exceeds Wasm32"))?;
        let mut result = Self(data);
        if result.word()? != DISPLAY_NAMES_WIRE_VERSION
            || result.word()? != u64::from(op.code()) * 2 + u64::from(response)
        {
            return Err(DisplayNamesWireError::Malformed(
                "version, operation, or direction",
            ));
        }
        Ok(result)
    }
    fn take(&mut self, size: usize) -> Result<&'a [u8], DisplayNamesWireError> {
        let (head, rest) = self
            .0
            .split_at_checked(size)
            .ok_or(DisplayNamesWireError::Malformed("truncated field"))?;
        self.0 = rest;
        Ok(head)
    }
    fn word(&mut self) -> Result<u64, DisplayNamesWireError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight bytes"),
        ))
    }
    fn domain<T>(&mut self, decode: fn(u64) -> Option<T>) -> Result<T, DisplayNamesWireError> {
        decode(self.word()?).ok_or(DisplayNamesWireError::Malformed("closed domain"))
    }
    fn count(&mut self, bytes_per_record: usize) -> Result<usize, DisplayNamesWireError> {
        let count = u32::try_from(self.word()?)
            .map_err(|_| DisplayNamesWireError::Malformed("count exceeds Wasm32"))?
            as usize;
        if count > self.0.len() / bytes_per_record {
            return Err(DisplayNamesWireError::Malformed(
                "count exceeds owned frame",
            ));
        }
        Ok(count)
    }
    fn text(&mut self) -> Result<&'a str, DisplayNamesWireError> {
        let size = u32::try_from(self.word()?)
            .map_err(|_| DisplayNamesWireError::Malformed("text exceeds Wasm32"))?
            as usize;
        core::str::from_utf8(self.take(size)?)
            .map_err(|_| DisplayNamesWireError::Malformed("invalid UTF8"))
    }
    fn locale(&mut self) -> Result<CanonicalLocaleId, DisplayNamesWireError> {
        CanonicalLocaleId::from_data(self.text()?)
            .map_err(|_| DisplayNamesWireError::Malformed("canonical locale spelling"))
    }
    fn locales(&mut self) -> Result<Box<[CanonicalLocaleId]>, DisplayNamesWireError> {
        let count = self.count(8)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| DisplayNamesWireError::Resource("locale allocation"))?;
        for _ in 0..count {
            result.push(self.locale()?);
        }
        Ok(result.into_boxed_slice())
    }
    fn configuration(
        &mut self,
        profiles: &DisplayNamesProfiles,
    ) -> Result<CheckedDisplayNamesConfiguration, DisplayNamesWireError> {
        let locale = profiles.admit(self.locale()?)?;
        let kind = self.domain(DisplayNamesType::from_wire_code)?;
        let style = self.domain(DisplayNamesStyle::from_wire_code)?;
        let fallback = self.domain(DisplayNamesFallback::from_wire_code)?;
        let language = self.word()?;
        let selection = match kind {
            DisplayNamesType::Language => DisplayNamesSelection::Language(
                DisplayNamesLanguageDisplay::from_wire_code(language)
                    .ok_or(DisplayNamesWireError::Malformed("languageDisplay"))?,
            ),
            DisplayNamesType::Region
            | DisplayNamesType::Script
            | DisplayNamesType::Currency
            | DisplayNamesType::Calendar
            | DisplayNamesType::DateTimeField => {
                if language != 0 {
                    return Err(DisplayNamesWireError::Malformed("nonlanguage slot"));
                }
                DisplayNamesSelection::from_options(kind, DisplayNamesLanguageDisplay::Dialect)
            }
        };
        Ok(CheckedDisplayNamesConfiguration::new(
            locale, selection, style, fallback,
        ))
    }
    fn utf16(&mut self) -> Result<Box<[u16]>, DisplayNamesWireError> {
        let size = u32::try_from(self.word()?)
            .map_err(|_| DisplayNamesWireError::Malformed("text exceeds Wasm32"))?
            as usize;
        let data = self.take(size)?;
        if data.len() % 2 != 0 {
            return Err(DisplayNamesWireError::Malformed("odd UTF16 byte length"));
        }
        let mut units = Vec::new();
        units
            .try_reserve_exact(data.len() / 2)
            .map_err(|_| DisplayNamesWireError::Resource("UTF16 allocation"))?;
        for pair in data.chunks_exact(2) {
            units.push(u16::from_le_bytes([pair[0], pair[1]]));
        }
        Ok(units.into_boxed_slice())
    }
    fn finish(self) -> Result<(), DisplayNamesWireError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(DisplayNamesWireError::Malformed("trailing bytes"))
        }
    }
}

pub fn encode_display_names_locale_request(
    request: &DisplayNamesLocaleRequest,
) -> Result<Vec<u8>, DisplayNamesWireError> {
    let mut writer = Writer::new(DisplayNamesWireOperation::ResolveLocale, false)?;
    writer.locales(&request.requested)?;
    writer.word(request.matcher.wire_code())?;
    Ok(writer.0)
}
pub fn decode_display_names_locale_request(
    data: &[u8],
) -> Result<DisplayNamesLocaleRequest, DisplayNamesWireError> {
    let mut reader = Reader::new(data, DisplayNamesWireOperation::ResolveLocale, false)?;
    let result = DisplayNamesLocaleRequest {
        requested: reader.locales()?,
        matcher: reader.domain(LocaleMatcher::from_wire_code)?,
    };
    reader.finish()?;
    Ok(result)
}
pub fn encode_display_names_locale_response(
    result: &ResolvedDisplayNamesLocale,
) -> Result<Vec<u8>, DisplayNamesWireError> {
    let mut writer = Writer::new(DisplayNamesWireOperation::ResolveLocale, true)?;
    writer.text(result.resolved().as_str())?;
    Ok(writer.0)
}
pub fn decode_display_names_locale_response(
    data: &[u8],
    profiles: &DisplayNamesProfiles,
) -> Result<ResolvedDisplayNamesLocale, DisplayNamesWireError> {
    let mut reader = Reader::new(data, DisplayNamesWireOperation::ResolveLocale, true)?;
    let result = profiles.admit(reader.locale()?)?;
    reader.finish()?;
    Ok(result)
}
pub fn encode_display_names_supported_request(
    request: &DisplayNamesLocaleRequest,
) -> Result<Vec<u8>, DisplayNamesWireError> {
    let mut writer = Writer::new(DisplayNamesWireOperation::SupportedLocales, false)?;
    writer.locales(&request.requested)?;
    writer.word(request.matcher.wire_code())?;
    Ok(writer.0)
}
pub fn decode_display_names_supported_request(
    data: &[u8],
) -> Result<DisplayNamesLocaleRequest, DisplayNamesWireError> {
    let mut reader = Reader::new(data, DisplayNamesWireOperation::SupportedLocales, false)?;
    let result = DisplayNamesLocaleRequest {
        requested: reader.locales()?,
        matcher: reader.domain(LocaleMatcher::from_wire_code)?,
    };
    reader.finish()?;
    Ok(result)
}
pub fn encode_display_names_supported_response(
    result: &DisplayNamesSupportedLocalesResult,
) -> Result<Vec<u8>, DisplayNamesWireError> {
    let mut writer = Writer::new(DisplayNamesWireOperation::SupportedLocales, true)?;
    writer.locales(&result.locales)?;
    Ok(writer.0)
}
pub fn decode_display_names_supported_response(
    data: &[u8],
) -> Result<DisplayNamesSupportedLocalesResult, DisplayNamesWireError> {
    let mut reader = Reader::new(data, DisplayNamesWireOperation::SupportedLocales, true)?;
    let result = DisplayNamesSupportedLocalesResult {
        locales: reader.locales()?,
    };
    reader.finish()?;
    if result
        .locales
        .iter()
        .enumerate()
        .any(|(index, locale)| result.locales[..index].contains(locale))
    {
        return Err(DisplayNamesWireError::Malformed(
            "duplicate supported locale",
        ));
    }
    Ok(result)
}
pub fn encode_display_name_request(
    request: &DisplayNameRequest,
) -> Result<Vec<u8>, DisplayNamesWireError> {
    let mut writer = Writer::new(DisplayNamesWireOperation::DisplayName, false)?;
    writer.configuration(request.configuration())?;
    writer.utf16(request.code())?;
    Ok(writer.0)
}
pub fn decode_display_name_request(
    data: &[u8],
    profiles: &DisplayNamesProfiles,
) -> Result<DisplayNameRequest, DisplayNamesWireError> {
    let mut reader = Reader::new(data, DisplayNamesWireOperation::DisplayName, false)?;
    let configuration = reader.configuration(profiles)?;
    let code = reader.utf16()?;
    reader.finish()?;
    Ok(DisplayNameRequest::new(configuration, code)?)
}
pub fn encode_display_name_response(
    result: &DisplayNameResult,
) -> Result<Vec<u8>, DisplayNamesWireError> {
    let mut writer = Writer::new(DisplayNamesWireOperation::DisplayName, true)?;
    match result.name() {
        None => writer.word(0)?,
        Some(name) => {
            writer.word(1)?;
            writer.text(name)?;
        }
    }
    Ok(writer.0)
}
pub fn decode_display_name_response(
    data: &[u8],
) -> Result<DisplayNameResult, DisplayNamesWireError> {
    let mut reader = Reader::new(data, DisplayNamesWireOperation::DisplayName, true)?;
    let name = match reader.word()? {
        0 => None,
        1 => Some(reader.text()?.into()),
        _ => return Err(DisplayNamesWireError::Malformed("optional-name presence")),
    };
    reader.finish()?;
    Ok(DisplayNameResult::new(name)?)
}

impl DisplayNamesWireOperation {
    pub const fn global_operation(self) -> crate::IntlHostOp {
        match self {
            Self::ResolveLocale => crate::IntlHostOp::ResolveDisplayNamesLocale,
            Self::SupportedLocales => crate::IntlHostOp::SupportedDisplayNamesLocales,
            Self::DisplayName => crate::IntlHostOp::DisplayName,
        }
    }
}
const _: () = {
    let mut index = 0;
    while index < DisplayNamesWireOperation::ALL.len() {
        let operation = DisplayNamesWireOperation::ALL[index];
        assert!(operation.code() == operation.global_operation().code());
        index += 1;
    }
};

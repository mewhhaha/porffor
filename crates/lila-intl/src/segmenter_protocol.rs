//! Three primitive Segmenter frames for the closed global operations33..35.
use crate::number_format::options::LocaleMatcher;
use crate::segmenter::*;
use crate::CanonicalLocaleId;
use core::fmt;

#[cfg(test)]
mod tests;
mod wire;
pub use wire::*;
pub const SEGMENTER_WIRE_VERSION: u64 = 1;
pub const SEGMENTER_HEADER_BYTES: u64 = 16;
pub const SEGMENTER_CONFIGURATION_WORDS: usize = 1;
const _: () = assert!(SEGMENTER_WIRE_VERSION == crate::number_protocol::NUMBER_WIRE_VERSION);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmenterConfigurationWord {
    Granularity,
}
impl SegmenterConfigurationWord {
    pub const ALL: [Self; 1] = [Self::Granularity];
    pub const fn index(self) -> usize {
        match self {
            Self::Granularity => 0,
        }
    }
    pub const fn offset(self) -> u64 {
        self.index() as u64 * 8
    }
}
impl CheckedSegmenterConfiguration {
    pub fn wire_words(&self) -> [u64; SEGMENTER_CONFIGURATION_WORDS] {
        [self.granularity().wire_code()]
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmenterWireOperation {
    ResolveLocale,
    SupportedLocales,
    SegmentUtf16,
}
impl SegmenterWireOperation {
    pub const ALL: [Self; 3] = [
        Self::ResolveLocale,
        Self::SupportedLocales,
        Self::SegmentUtf16,
    ];
    pub const fn global_operation(self) -> crate::IntlHostOp {
        match self {
            Self::ResolveLocale => crate::IntlHostOp::ResolveSegmenterLocale,
            Self::SupportedLocales => crate::IntlHostOp::SupportedSegmenterLocales,
            Self::SegmentUtf16 => crate::IntlHostOp::SegmentUtf16,
        }
    }
    pub const fn code(self) -> u16 {
        match self {
            Self::ResolveLocale => 33,
            Self::SupportedLocales => 34,
            Self::SegmentUtf16 => 35,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SegmenterWireError {
    Malformed(&'static str),
    Native(SegmenterError),
    Resource(&'static str),
}
impl From<SegmenterError> for SegmenterWireError {
    fn from(value: SegmenterError) -> Self {
        Self::Native(value)
    }
}
impl fmt::Display for SegmenterWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(reason) => write!(f, "invalid Segmenter frame: {reason}"),
            Self::Native(error) => error.fmt(f),
            Self::Resource(reason) => write!(f, "Segmenter wire resource: {reason}"),
        }
    }
}
impl std::error::Error for SegmenterWireError {}

struct Writer(Vec<u8>);
impl Writer {
    fn new(op: SegmenterWireOperation, response: bool) -> Result<Self, SegmenterWireError> {
        let mut result = Self(Vec::new());
        result.word(SEGMENTER_WIRE_VERSION)?;
        result.word(u64::from(op.code()) * 2 + u64::from(response))?;
        Ok(result)
    }
    fn append(&mut self, data: &[u8]) -> Result<(), SegmenterWireError> {
        let size = self
            .0
            .len()
            .checked_add(data.len())
            .ok_or(SegmenterWireError::Resource("extent"))?;
        u32::try_from(size).map_err(|_| SegmenterWireError::Resource("frame exceeds Wasm32"))?;
        self.0
            .try_reserve(data.len())
            .map_err(|_| SegmenterWireError::Resource("allocation"))?;
        self.0.extend_from_slice(data);
        Ok(())
    }
    fn word(&mut self, value: u64) -> Result<(), SegmenterWireError> {
        self.append(&value.to_le_bytes())
    }
    fn text(&mut self, text: &str) -> Result<(), SegmenterWireError> {
        self.word(text.len() as u64)?;
        self.append(text.as_bytes())
    }
    fn locales(&mut self, locales: &[CanonicalLocaleId]) -> Result<(), SegmenterWireError> {
        self.word(locales.len() as u64)?;
        for locale in locales {
            self.text(locale.as_str())?;
        }
        Ok(())
    }
}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn new(
        data: &'a [u8],
        op: SegmenterWireOperation,
        response: bool,
    ) -> Result<Self, SegmenterWireError> {
        u32::try_from(data.len())
            .map_err(|_| SegmenterWireError::Resource("frame exceeds Wasm32"))?;
        let mut result = Self(data);
        if result.word()? != SEGMENTER_WIRE_VERSION
            || result.word()? != u64::from(op.code()) * 2 + u64::from(response)
        {
            return Err(SegmenterWireError::Malformed(
                "version, operation, or direction",
            ));
        }
        Ok(result)
    }
    fn take(&mut self, size: usize) -> Result<&'a [u8], SegmenterWireError> {
        let (head, rest) = self
            .0
            .split_at_checked(size)
            .ok_or(SegmenterWireError::Malformed("truncated field"))?;
        self.0 = rest;
        Ok(head)
    }
    fn word(&mut self) -> Result<u64, SegmenterWireError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight bytes"),
        ))
    }
    fn count(&mut self, bytes_per_row: usize) -> Result<usize, SegmenterWireError> {
        let count = usize::try_from(self.word()?)
            .map_err(|_| SegmenterWireError::Malformed("count extent"))?;
        if count > self.0.len() / bytes_per_row {
            return Err(SegmenterWireError::Malformed(
                "count exceeds remaining frame",
            ));
        }
        Ok(count)
    }
    fn text(&mut self) -> Result<&'a str, SegmenterWireError> {
        let size = self.count(1)?;
        core::str::from_utf8(self.take(size)?)
            .map_err(|_| SegmenterWireError::Malformed("UTF8 locale"))
    }
    fn locale(&mut self) -> Result<CanonicalLocaleId, SegmenterWireError> {
        CanonicalLocaleId::from_data(self.text()?)
            .map_err(|_| SegmenterWireError::Malformed("noncanonical locale"))
    }
    fn locales(&mut self) -> Result<Box<[CanonicalLocaleId]>, SegmenterWireError> {
        let count = self.count(8)?;
        let mut rows = Vec::new();
        rows.try_reserve_exact(count)
            .map_err(|_| SegmenterWireError::Resource("locale allocation"))?;
        for _ in 0..count {
            let locale = self.locale()?;
            if rows.contains(&locale) {
                return Err(SegmenterWireError::Malformed("duplicate locale"));
            }
            rows.push(locale);
        }
        Ok(rows.into_boxed_slice())
    }
    fn finish(self) -> Result<(), SegmenterWireError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(SegmenterWireError::Malformed("trailing bytes"))
        }
    }
}

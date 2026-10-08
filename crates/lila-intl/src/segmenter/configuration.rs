use super::*;

#[derive(Debug, Clone)]
pub struct CheckedSegmenterConfiguration {
    locale: ResolvedSegmenterLocale,
    granularity: SegmenterGranularity,
}
impl CheckedSegmenterConfiguration {
    pub const fn new(locale: ResolvedSegmenterLocale, granularity: SegmenterGranularity) -> Self {
        Self {
            locale,
            granularity,
        }
    }
    pub fn locale(&self) -> &ResolvedSegmenterLocale {
        &self.locale
    }
    pub const fn granularity(&self) -> SegmenterGranularity {
        self.granularity
    }
}

/// Original JavaScript UTF16 units. Isolated surrogates are retained, rather
/// than becoming replacement characters through a UTF8 conversion.
#[derive(Debug, Clone)]
pub struct SegmentUtf16Request {
    configuration: CheckedSegmenterConfiguration,
    input: Arc<[u16]>,
}
impl SegmentUtf16Request {
    pub fn new(
        configuration: CheckedSegmenterConfiguration,
        input: Box<[u16]>,
    ) -> Result<Self, SegmenterError> {
        let extent = input
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(configuration.locale().resolved().as_str().len()))
            .and_then(|n| n.checked_add(56))
            .ok_or(SegmenterError::Resource("request extent"))?;
        u32::try_from(extent).map_err(|_| SegmenterError::Resource("request exceeds Wasm32"))?;
        Ok(Self {
            configuration,
            input: Arc::from(input),
        })
    }
    pub fn configuration(&self) -> &CheckedSegmenterConfiguration {
        &self.configuration
    }
    pub fn input(&self) -> &[u16] {
        &self.input
    }
    pub(super) fn shared_input(&self) -> Arc<[u16]> {
        Arc::clone(&self.input)
    }
}

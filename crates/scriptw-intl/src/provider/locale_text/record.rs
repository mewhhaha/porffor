use crate::CanonicalLocaleId;

/// Unknown or non-general direction is represented by `None`, never a default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum LocaleTextDirection {
    #[serde(rename = "ltr")]
    LeftToRight,
    #[serde(rename = "rtl")]
    RightToLeft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleTextInfoRequest {
    locale: CanonicalLocaleId,
}

impl LocaleTextInfoRequest {
    #[must_use]
    pub const fn new(locale: CanonicalLocaleId) -> Self {
        Self { locale }
    }

    #[must_use]
    pub fn into_locale(self) -> CanonicalLocaleId {
        self.locale
    }
}

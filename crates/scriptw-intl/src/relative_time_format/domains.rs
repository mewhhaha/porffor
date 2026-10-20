use super::RelativeTimeError;
use serde::Deserialize;

macro_rules! domain {
    ($name:ident { $($variant:ident = $code:literal => $text:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
        pub enum $name { $(#[serde(rename = $text)] $variant),+ }
        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            pub const OPTIONS: &'static [(&'static str, i64)] = &[$(($text, $code)),+];
            pub const fn name(self) -> &'static str { match self { $(Self::$variant => $text),+ } }
            pub const fn wire_code(self) -> u64 { match self { $(Self::$variant => $code),+ } }
            pub const fn from_wire_code(code: u64) -> Option<Self> {
                match code { $($code => Some(Self::$variant),)+ _ => None }
            }
            pub fn parse(value: &str) -> Option<Self> {
                match value { $($text => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}
domain!(RelativeUnit {
    Year = 1 => "year", Quarter = 2 => "quarter", Month = 3 => "month", Week = 4 => "week",
    Day = 5 => "day", Hour = 6 => "hour", Minute = 7 => "minute", Second = 8 => "second"
});
domain!(RelativeStyle { Long = 1 => "long", Short = 2 => "short", Narrow = 3 => "narrow" });
domain!(RelativeNumeric { Always = 1 => "always", Auto = 2 => "auto" });

impl RelativeUnit {
    pub(super) const fn index(self) -> usize {
        self.wire_code() as usize - 1
    }
    /// Singular and plural spellings are the complete sanctioned unit domain.
    pub fn from_observed(value: &str) -> Option<Self> {
        Self::parse(value).or_else(|| value.strip_suffix('s').and_then(Self::parse))
    }
}
impl RelativeStyle {
    pub(super) const fn index(self) -> usize {
        self.wire_code() as usize - 1
    }
}

/// IEEE bits retain -0 and guarantee that no operation can receive NaN/infinity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FiniteRelativeNumber(u64);
impl FiniteRelativeNumber {
    pub fn new(value: f64) -> Result<Self, RelativeTimeError> {
        if value.is_finite() {
            Ok(Self(value.to_bits()))
        } else {
            Err(RelativeTimeError::NonFinite)
        }
    }
    pub fn from_bits(bits: u64) -> Result<Self, RelativeTimeError> {
        Self::new(f64::from_bits(bits))
    }
    pub const fn bits(self) -> u64 {
        self.0
    }
    pub(super) fn value(self) -> f64 {
        f64::from_bits(self.0)
    }
    pub(super) const fn past(self) -> bool {
        self.0 >> 63 != 0
    }
    pub(super) fn auto_offset(self) -> Option<i8> {
        let value = self.value();
        [-2, -1, 0, 1, 2]
            .into_iter()
            .find(|offset| value == f64::from(*offset))
    }
}

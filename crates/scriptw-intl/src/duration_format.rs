//! Checked image-backed DurationFormat; JavaScript observations remain in Wasm.
use core::fmt;
mod configuration;
mod partition;
mod profile_identity;
mod profiles;
mod raw;
mod record;
#[cfg(test)]
mod tests;
pub use configuration::{
    CheckedDurationConfiguration, DurationOptions, DurationUnitOption, ResolvedDurationLocale,
};
pub(crate) use profile_identity::{DURATION_FORMAT_DATA_SHA256, DURATION_PROFILE_SHA256};
pub use profiles::{embedded_duration_profiles, DurationProfiles};
pub use record::DurationRecord;

macro_rules! domain {
    ($name:ident {$($variant:ident => $text:literal),+ $(,)?}) => {
        #[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
        pub enum $name {$($variant),+}
        impl $name {
            pub const ALL: &'static [Self]=&[$(Self::$variant),+];
            pub const fn name(self)->&'static str {match self {$(Self::$variant=>$text),+}}
            pub fn parse(text:&str)->Option<Self>{match text {$($text=>Some(Self::$variant),)+ _=>None}}
            pub const fn index(self)->usize{self as usize}
        }
    }
}
domain!(DurationUnit {Year=>"year",Month=>"month",Week=>"week",Day=>"day",Hour=>"hour",Minute=>"minute",Second=>"second",Millisecond=>"millisecond",Microsecond=>"microsecond",Nanosecond=>"nanosecond"});
domain!(DurationStyle {Long=>"long",Short=>"short",Narrow=>"narrow",Digital=>"digital"});
domain!(DurationUnitStyle {Long=>"long",Short=>"short",Narrow=>"narrow",Numeric=>"numeric",TwoDigit=>"2-digit"});
domain!(DurationDisplay {Auto=>"auto",Always=>"always"});
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurationFractionalDigits(u8);
impl DurationFractionalDigits {
    pub fn new(value: u8) -> Result<Self, DurationError> {
        if value <= 9 {
            Ok(Self(value))
        } else {
            Err(DurationError::InvalidOptions)
        }
    }
    pub const fn value(self) -> u8 {
        self.0
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationSupportedLocalesRequest {
    pub requested: Box<[crate::CanonicalLocaleId]>,
    pub matcher: crate::number_format::options::LocaleMatcher,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurationError {
    UnavailableService(crate::IntlService),
    InvalidRecord,
    InvalidBounds,
    InvalidOptions,
    InvalidLocale,
    InvalidProfile,
    Resource(&'static str),
    Number(Box<str>),
    List(Box<str>),
}
impl fmt::Display for DurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::InvalidRecord
            | Self::InvalidBounds
            | Self::InvalidOptions
            | Self::InvalidLocale
            | Self::InvalidProfile
            | Self::Resource(_)
            | Self::Number(_)
            | Self::List(_) => write!(f, "DurationFormat: {self:?}"),
        }
    }
}
impl std::error::Error for DurationError {}
impl From<crate::number_format::NumberFormatKernelError> for DurationError {
    fn from(e: crate::number_format::NumberFormatKernelError) -> Self {
        Self::Number(e.to_string().into_boxed_str())
    }
}
impl From<crate::list_format::ListFormatOperationError> for DurationError {
    fn from(e: crate::list_format::ListFormatOperationError) -> Self {
        Self::List(e.to_string().into_boxed_str())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationPart {
    part: crate::number_format::NumberPart,
    unit: Option<DurationUnit>,
}
impl DurationPart {
    pub fn text(&self) -> &str {
        self.part.text()
    }
    pub const fn kind(&self) -> crate::number_format::NumberPartKind {
        self.part.kind()
    }
    pub const fn unit(&self) -> Option<DurationUnit> {
        self.unit
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationPartition {
    parts: Box<[DurationPart]>,
    bytes: usize,
}
impl DurationPartition {
    pub fn parts(&self) -> &[DurationPart] {
        &self.parts
    }
    pub fn to_text(&self) -> Result<String, DurationError> {
        let mut out = String::new();
        out.try_reserve_exact(self.bytes)
            .map_err(|_| DurationError::Resource("text allocation"))?;
        for p in &self.parts {
            out.push_str(p.text())
        }
        Ok(out)
    }
}
pub fn format_duration_parts(
    configuration: &CheckedDurationConfiguration,
    record: &DurationRecord,
    profiles: &DurationProfiles,
    limits: &crate::number_format::PartitionLimits,
) -> Result<DurationPartition, DurationError> {
    profiles.format_parts(configuration, record, limits)
}

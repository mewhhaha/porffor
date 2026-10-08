//! Checked public service admission and its actual immutable frame closure.
use crate::{EmptyIntlProfile, IntlService, IntlServiceSet};
pub const INTL_SERVICE_SELECTION_CUSTOM_SECTION: &str = "lila.intl.services.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum IntlDataComponent {
    Locale = 0,
    List = 1,
    Collator = 2,
    Number = 3,
    Segmenter = 4,
    DisplayNames = 5,
    RelativeTime = 6,
    Duration = 7,
    NamedTimeZones = 8,
    DateTime = 9,
    TimeZoneNames = 10,
    LocaleInformation = 11,
}
impl IntlDataComponent {
    pub const ALL: [Self; 12] = [
        Self::Locale,
        Self::List,
        Self::Collator,
        Self::Number,
        Self::Segmenter,
        Self::DisplayNames,
        Self::RelativeTime,
        Self::Duration,
        Self::NamedTimeZones,
        Self::DateTime,
        Self::TimeZoneNames,
        Self::LocaleInformation,
    ];
    pub const fn code(self) -> u16 {
        self as u16
    }
    pub fn from_code(code: u16) -> Option<Self> {
        Self::ALL.get(usize::from(code)).copied()
    }
    pub const fn section_name(self) -> &'static str {
        match self {
            Self::Locale => crate::INTL_LOCALE_DATA_CUSTOM_SECTION,
            Self::List => crate::INTL_LIST_DATA_CUSTOM_SECTION,
            Self::Collator => crate::INTL_COLLATOR_DATA_CUSTOM_SECTION,
            Self::Number => crate::INTL_NUMBER_DATA_CUSTOM_SECTION,
            Self::Segmenter => crate::INTL_SEGMENTER_DATA_CUSTOM_SECTION,
            Self::DisplayNames => crate::INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION,
            Self::RelativeTime => crate::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION,
            Self::Duration => crate::INTL_DURATION_DATA_CUSTOM_SECTION,
            Self::NamedTimeZones => crate::INTL_NAMED_TIME_ZONE_DATA_CUSTOM_SECTION,
            Self::DateTime => crate::INTL_DATETIME_DATA_CUSTOM_SECTION,
            Self::TimeZoneNames => crate::INTL_TIME_ZONE_NAMES_DATA_CUSTOM_SECTION,
            Self::LocaleInformation => crate::INTL_NATIVE_LOCALE_INFORMATION_CUSTOM_SECTION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntlDataComponentSet(u16);
impl IntlDataComponentSet {
    const fn with(self, component: IntlDataComponent) -> Self {
        Self(self.0 | (1 << component.code()))
    }
    pub const fn contains(self, component: IntlDataComponent) -> bool {
        self.0 & (1 << component.code()) != 0
    }
    pub fn iter(self) -> impl Iterator<Item = IntlDataComponent> {
        IntlDataComponent::ALL
            .into_iter()
            .filter(move |component| self.contains(*component))
    }
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }
}

/// Only requested services can authorize public operations. Retaining a shared
/// Number or List foundation never advertises its formatter as a service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedIntlServiceSelection {
    requested: IntlServiceSet,
    components: IntlDataComponentSet,
}
impl CheckedIntlServiceSelection {
    pub fn new(requested: IntlServiceSet) -> Result<Self, EmptyIntlProfile> {
        if requested.is_empty() {
            return Err(EmptyIntlProfile);
        }
        use IntlDataComponent as C;
        let mut components = IntlDataComponentSet(0).with(C::Locale);
        for service in requested.iter() {
            let required: &[C] = match service {
                IntlService::Locale => &[
                    C::List,
                    C::Number,
                    C::Collator,
                    C::NamedTimeZones,
                    C::DateTime,
                    C::LocaleInformation,
                ],
                IntlService::Collator => &[C::Collator],
                IntlService::NumberFormat | IntlService::PluralRules => &[C::List, C::Number],
                IntlService::RelativeTimeFormat => &[C::List, C::Number, C::RelativeTime],
                IntlService::DurationFormat => &[C::List, C::Number, C::Duration],
                IntlService::ListFormat => &[C::List],
                IntlService::DisplayNames => &[C::DisplayNames],
                IntlService::Segmenter => &[C::Segmenter],
                IntlService::DateTimeFormat => &[C::NamedTimeZones, C::DateTime, C::TimeZoneNames],
            };
            for component in required {
                components = components.with(*component);
            }
        }
        Ok(Self {
            requested,
            components,
        })
    }
    pub const fn requested(&self) -> IntlServiceSet {
        self.requested
    }
    pub const fn components(&self) -> IntlDataComponentSet {
        self.components
    }
    pub const fn contains_component(&self, component: IntlDataComponent) -> bool {
        self.components.contains(component)
    }
    pub const fn permits(&self, service: IntlService) -> bool {
        self.requested.contains(service)
    }
    pub fn wire(&self) -> u16 {
        self.requested
            .iter()
            .fold(0, |bits, service| bits | (1 << service as u8))
    }
    pub fn from_wire(bits: u16) -> Result<Self, InvalidIntlServiceSelection> {
        let allowed = (1u16 << IntlService::ALL.len()) - 1;
        if bits == 0 || bits & !allowed != 0 {
            return Err(InvalidIntlServiceSelection);
        }
        let requested = IntlService::ALL
            .iter()
            .copied()
            .filter(|service| bits & (1 << *service as u8) != 0)
            .collect();
        Self::new(requested).map_err(|_| InvalidIntlServiceSelection)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidIntlServiceSelection;
impl core::fmt::Display for InvalidIntlServiceSelection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("invalid or empty Custom Intl service selection")
    }
}
impl std::error::Error for InvalidIntlServiceSelection {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnavailableIntlService(pub IntlService);
impl core::fmt::Display for UnavailableIntlService {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Intl {} service is unavailable in the selected Custom data",
            self.0.name()
        )
    }
}
impl std::error::Error for UnavailableIntlService {}

macro_rules! unavailable_error {
    ($($error:ty),+ $(,)?) => { $(impl From<UnavailableIntlService> for $error {
        fn from(error: UnavailableIntlService) -> Self { Self::UnavailableService(error.0) }
    })+ };
}
unavailable_error!(
    crate::CollatorOperationError,
    crate::DisplayNamesError,
    crate::RelativeTimeError,
    crate::NumberFormatOperationError,
    crate::PluralRulesOperationError,
    crate::ListFormatOperationError,
    crate::DurationError,
    crate::SegmenterError,
    crate::DateTimeFormatError,
    crate::NamedTimeZoneDataError,
    crate::NamedTimeZoneLookupError,
    crate::TimeZoneResolveError,
    crate::LocaleCalendarsError,
    crate::LocaleCollationsError,
    crate::LocaleTimeZonesError,
    crate::LocaleNumberingSystemsError,
    crate::LocaleHourCyclesError,
    crate::LocaleTextError,
    crate::LocaleWeekError
);
impl From<UnavailableIntlService> for crate::NumberWireError {
    fn from(error: UnavailableIntlService) -> Self {
        Self::UnavailableService(error.0)
    }
}
macro_rules! unavailable_wire {
    ($(($wire:ty, $variant:ident)),+ $(,)?) => { $(impl From<UnavailableIntlService> for $wire {
        fn from(error: UnavailableIntlService) -> Self { Self::$variant(error.into()) }
    })+ };
}
unavailable_wire!(
    (crate::DisplayNamesWireError, Native),
    (crate::SegmenterWireError, Native),
    (crate::DurationWireError, Rejected),
    (crate::ListWireError, Operation),
    (crate::CollatorWireError, Operation),
    (crate::PluralWireError, Operation),
    (crate::RelativeWireError, Operation)
);

#[cfg(test)]
mod tests;

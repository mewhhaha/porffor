#[cfg(test)]
use icu_calendar::AnyCalendar;
use icu_calendar::AnyCalendarKind;

use crate::datetime::{DateTimeCalendar, DateTimeFormatError};

/// Calculation identity; sharing a CLDR name pool never changes this identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::provider::datetime) enum CalendarId {
    Buddhist,
    Chinese,
    Coptic,
    Dangi,
    Ethioaa,
    Ethiopic,
    Gregory,
    Hebrew,
    Indian,
    IslamicCivil,
    IslamicTbla,
    IslamicUmalqura,
    Iso8601,
    Japanese,
    Persian,
    Roc,
}

impl CalendarId {
    pub(in crate::provider::datetime) const ALL: [Self; 16] = [
        Self::Buddhist,
        Self::Chinese,
        Self::Coptic,
        Self::Dangi,
        Self::Ethioaa,
        Self::Ethiopic,
        Self::Gregory,
        Self::Hebrew,
        Self::Indian,
        Self::IslamicCivil,
        Self::IslamicTbla,
        Self::IslamicUmalqura,
        Self::Iso8601,
        Self::Japanese,
        Self::Persian,
        Self::Roc,
    ];

    pub(in crate::provider::datetime) const fn as_str(self) -> &'static str {
        match self {
            Self::Buddhist => "buddhist",
            Self::Chinese => "chinese",
            Self::Coptic => "coptic",
            Self::Dangi => "dangi",
            Self::Ethioaa => "ethioaa",
            Self::Ethiopic => "ethiopic",
            Self::Gregory => "gregory",
            Self::Hebrew => "hebrew",
            Self::Indian => "indian",
            Self::IslamicCivil => "islamic-civil",
            Self::IslamicTbla => "islamic-tbla",
            Self::IslamicUmalqura => "islamic-umalqura",
            Self::Iso8601 => "iso8601",
            Self::Japanese => "japanese",
            Self::Persian => "persian",
            Self::Roc => "roc",
        }
    }

    #[cfg(test)]
    pub(in crate::provider::datetime) fn from_canonical(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|calendar| calendar.as_str() == value)
    }

    pub(in crate::provider::datetime) fn from_admitted(value: DateTimeCalendar) -> Self {
        // New public variants require an explicit calculation association.
        match value {
            DateTimeCalendar::Gregorian => Self::Gregory,
            DateTimeCalendar::Iso8601 => Self::Iso8601,
            DateTimeCalendar::Chinese => Self::Chinese,
            DateTimeCalendar::Buddhist => Self::Buddhist,
            DateTimeCalendar::Coptic => Self::Coptic,
            DateTimeCalendar::Dangi => Self::Dangi,
            DateTimeCalendar::Ethioaa => Self::Ethioaa,
            DateTimeCalendar::Ethiopic => Self::Ethiopic,
            DateTimeCalendar::Hebrew => Self::Hebrew,
            DateTimeCalendar::Indian => Self::Indian,
            DateTimeCalendar::IslamicCivil => Self::IslamicCivil,
            DateTimeCalendar::IslamicTbla => Self::IslamicTbla,
            DateTimeCalendar::IslamicUmalqura => Self::IslamicUmalqura,
            DateTimeCalendar::Japanese => Self::Japanese,
            DateTimeCalendar::Persian => Self::Persian,
            DateTimeCalendar::Roc => Self::Roc,
        }
    }

    pub(in crate::provider::datetime) const fn kind(self) -> AnyCalendarKind {
        match self {
            Self::Buddhist => AnyCalendarKind::Buddhist,
            Self::Chinese => AnyCalendarKind::Chinese,
            Self::Coptic => AnyCalendarKind::Coptic,
            Self::Dangi => AnyCalendarKind::Dangi,
            Self::Ethioaa => AnyCalendarKind::EthiopianAmeteAlem,
            Self::Ethiopic => AnyCalendarKind::Ethiopian,
            Self::Gregory => AnyCalendarKind::Gregorian,
            Self::Hebrew => AnyCalendarKind::Hebrew,
            Self::Indian => AnyCalendarKind::Indian,
            Self::IslamicCivil => AnyCalendarKind::HijriTabularTypeIIFriday,
            Self::IslamicTbla => AnyCalendarKind::HijriTabularTypeIIThursday,
            Self::IslamicUmalqura => AnyCalendarKind::HijriUmmAlQura,
            Self::Iso8601 => AnyCalendarKind::Iso,
            Self::Japanese => AnyCalendarKind::Japanese,
            Self::Persian => AnyCalendarKind::Persian,
            Self::Roc => AnyCalendarKind::Roc,
        }
    }

    pub(in crate::provider::datetime) const fn index(self) -> usize {
        match self {
            Self::Buddhist => 0,
            Self::Chinese => 1,
            Self::Coptic => 2,
            Self::Dangi => 3,
            Self::Ethioaa => 4,
            Self::Ethiopic => 5,
            Self::Gregory => 6,
            Self::Hebrew => 7,
            Self::Indian => 8,
            Self::IslamicCivil => 9,
            Self::IslamicTbla => 10,
            Self::IslamicUmalqura => 11,
            Self::Iso8601 => 12,
            Self::Japanese => 13,
            Self::Persian => 14,
            Self::Roc => 15,
        }
    }

    #[cfg(test)]
    pub(in crate::provider::datetime) fn new(self) -> std::sync::Arc<AnyCalendar> {
        super::kernels::pinned().get(self)
    }
}

/// ICU era codes translated once; this is not ICU's unrelated `era_index`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::provider::datetime) enum EraName {
    BeforeCommon,
    Common,
    Buddhist,
    Coptic,
    AmeteAlem,
    AmeteMihret,
    Hebrew,
    Shaka,
    BeforeHijrah,
    Hijrah,
    Meiji,
    Taisho,
    Showa,
    Heisei,
    Reiwa,
    Persian,
    BeforeRoc,
    Roc,
}

impl EraName {
    pub(in crate::provider::datetime) fn from_native(
        calendar: CalendarId,
        code: &str,
    ) -> Result<Self, DateTimeFormatError> {
        use CalendarId as C;
        match (calendar, code) {
            (C::Gregory | C::Iso8601 | C::Japanese, "bce") => Ok(Self::BeforeCommon),
            (C::Gregory | C::Iso8601 | C::Japanese, "ce") => Ok(Self::Common),
            (C::Buddhist, "be") => Ok(Self::Buddhist),
            (C::Coptic, "am") => Ok(Self::Coptic),
            (C::Ethioaa | C::Ethiopic, "aa") => Ok(Self::AmeteAlem),
            (C::Ethiopic, "am") => Ok(Self::AmeteMihret),
            (C::Hebrew, "am") => Ok(Self::Hebrew),
            (C::Indian, "shaka") => Ok(Self::Shaka),
            (C::IslamicCivil | C::IslamicTbla | C::IslamicUmalqura, "bh") => Ok(Self::BeforeHijrah),
            (C::IslamicCivil | C::IslamicTbla | C::IslamicUmalqura, "ah") => Ok(Self::Hijrah),
            (C::Japanese, "meiji") => Ok(Self::Meiji),
            (C::Japanese, "taisho") => Ok(Self::Taisho),
            (C::Japanese, "showa") => Ok(Self::Showa),
            (C::Japanese, "heisei") => Ok(Self::Heisei),
            (C::Japanese, "reiwa") => Ok(Self::Reiwa),
            (C::Persian, "ap") => Ok(Self::Persian),
            (C::Roc, "broc") => Ok(Self::BeforeRoc),
            (C::Roc, "roc") => Ok(Self::Roc),
            _ => Err(DateTimeFormatError::InvalidProfile(
                "unexpected native calendar era".into(),
            )),
        }
    }
}

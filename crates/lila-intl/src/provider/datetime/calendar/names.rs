use super::{CalendarId, EraName, Month};
use crate::datetime::DateTimeFormatError;
use crate::provider::datetime::{
    names::{EraSource, MonthYearType, NameKey},
    pattern::{NameContext, NameWidth},
    profile::invalid,
};

impl CalendarId {
    pub(in crate::provider::datetime) const fn eras(self) -> &'static [EraName] {
        use EraName as E;
        match self {
            Self::Gregory | Self::Iso8601 => &[E::BeforeCommon, E::Common],
            Self::Buddhist => &[E::Buddhist],
            Self::Chinese | Self::Dangi => &[],
            Self::Coptic => &[E::Coptic],
            Self::Ethioaa => &[E::AmeteAlem],
            Self::Ethiopic => &[E::AmeteAlem, E::AmeteMihret],
            Self::Hebrew => &[E::Hebrew],
            Self::Indian => &[E::Shaka],
            Self::IslamicCivil | Self::IslamicTbla | Self::IslamicUmalqura => {
                &[E::BeforeHijrah, E::Hijrah]
            }
            // The two Gregorian keys have their own sourced namespace before1873.
            Self::Japanese => &[
                E::BeforeCommon,
                E::Common,
                E::Meiji,
                E::Taisho,
                E::Showa,
                E::Heisei,
                E::Reiwa,
            ],
            Self::Persian => &[E::Persian],
            Self::Roc => &[E::BeforeRoc, E::Roc],
        }
    }

    pub(in crate::provider::datetime) const fn is_cyclic(self) -> bool {
        matches!(self, Self::Chinese | Self::Dangi)
    }

    pub(in crate::provider::datetime) const fn month_names(self) -> u8 {
        match self {
            Self::Coptic | Self::Ethioaa | Self::Ethiopic | Self::Hebrew => 13,
            Self::Buddhist
            | Self::Chinese
            | Self::Dangi
            | Self::Gregory
            | Self::Indian
            | Self::IslamicCivil
            | Self::IslamicTbla
            | Self::IslamicUmalqura
            | Self::Iso8601
            | Self::Japanese
            | Self::Persian
            | Self::Roc => 12,
        }
    }
}

impl EraName {
    pub(in crate::provider::datetime) fn key(
        self,
        calendar: CalendarId,
        width: NameWidth,
    ) -> Result<NameKey, DateTimeFormatError> {
        if !calendar.eras().contains(&self) {
            return Err(invalid("era name disagrees with calculation calendar"));
        }
        // These are genuine CLDR record indices, never ICU era_index values.
        let index = match self {
            Self::BeforeCommon
            | Self::Buddhist
            | Self::AmeteAlem
            | Self::Hebrew
            | Self::Shaka
            | Self::Hijrah
            | Self::Persian
            | Self::BeforeRoc => 0,
            Self::Common | Self::Coptic | Self::AmeteMihret | Self::BeforeHijrah | Self::Roc => 1,
            Self::Meiji => 232,
            Self::Taisho => 233,
            Self::Showa => 234,
            Self::Heisei => 235,
            Self::Reiwa => 236,
        };
        let source = if calendar == CalendarId::Japanese
            && matches!(self, Self::BeforeCommon | Self::Common)
        {
            Some(EraSource::Gregorian)
        } else {
            None
        };
        Ok(NameKey::Era(width, index, source))
    }
}

impl Month {
    pub(in crate::provider::datetime) const fn name_index(self, calendar: CalendarId) -> u8 {
        let code = self.formatting();
        if matches!(calendar, CalendarId::Hebrew) && (code.number() >= 6 || code.is_leap()) {
            // CLDR Hebrew Adar is month7 in ordinary years, which skip month6.
            // M05L is Adar I/month6; M06L selects month7's leap-year name.
            code.number() + 1
        } else {
            code.number()
        }
    }

    pub(in crate::provider::datetime) fn key(
        self,
        calendar: CalendarId,
        context: NameContext,
        width: NameWidth,
    ) -> NameKey {
        let year_type = if calendar == CalendarId::Hebrew
            && self.formatting().number() == 6
            && self.formatting().is_leap()
        {
            Some(MonthYearType::Leap)
        } else {
            None
        };
        NameKey::Month(context, width, self.name_index(calendar), year_type)
    }

    pub(in crate::provider::datetime) const fn uses_leap_placeholder(
        self,
        calendar: CalendarId,
    ) -> bool {
        calendar.is_cyclic() && self.formatting().is_leap()
    }
}

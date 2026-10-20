//! Public and dependent readers borrow one admitted physical Number owner.

use super::*;

#[derive(Debug)]
pub(crate) enum NumberLocaleDomains {
    Complete,
    Projected {
        public: Box<[Box<str>]>,
        relative_time: Box<[Box<str>]>,
        duration: Box<[Box<str>]>,
        defaults: Box<[(Box<str>, u8)]>,
        currency_codes: Option<Box<[CurrencyCode]>>,
        numbering_systems: Option<Box<[crate::number_format::NumberingSystemOption]>>,
    },
}

#[derive(Debug, Clone, Copy)]
enum NumberLocalePurpose {
    Public,
    RelativeTime,
    Duration,
}

/// The catalogue cannot be supplied separately from its actual Arc owner.
#[derive(Clone, Copy)]
pub(crate) struct NumberLocaleView<'a> {
    owner: &'a Arc<NumberProfiles>,
    purpose: NumberLocalePurpose,
}

impl<'a> NumberLocaleView<'a> {
    pub(crate) fn public(owner: &'a Arc<NumberProfiles>) -> Self {
        Self {
            owner,
            purpose: NumberLocalePurpose::Public,
        }
    }

    pub(crate) fn relative(owner: &'a Arc<NumberProfiles>) -> Self {
        Self {
            owner,
            purpose: NumberLocalePurpose::RelativeTime,
        }
    }

    pub(crate) fn duration(owner: &'a Arc<NumberProfiles>) -> Self {
        Self {
            owner,
            purpose: NumberLocalePurpose::Duration,
        }
    }

    pub(crate) fn owner(self) -> &'a Arc<NumberProfiles> {
        self.owner
    }

    pub(crate) fn locales(self) -> &'a [Box<str>] {
        match &self.owner.domains {
            NumberLocaleDomains::Complete => &self.owner.locales,
            NumberLocaleDomains::Projected {
                public,
                relative_time,
                duration,
                ..
            } => match self.purpose {
                NumberLocalePurpose::Public => public,
                NumberLocalePurpose::RelativeTime => relative_time,
                NumberLocalePurpose::Duration => duration,
            },
        }
    }

    pub(crate) fn contains(self, locale: &str) -> bool {
        self.locales()
            .binary_search_by(|name| name.as_ref().cmp(locale))
            .is_ok()
    }
}

impl NumberProfiles {
    /// Locale matching uses the same admitted authority as the selected image,
    /// never a host default or an independently installed baked provider.
    pub(crate) fn maximize_for_matching(&self, locale: &mut icu_locale::LanguageIdentifier) {
        self.locale_data.expander().maximize(locale);
    }

    pub(crate) fn uses_locale(&self, locale: &crate::LocaleDataImage) -> bool {
        self.locale_data.same_owner(locale)
    }

    pub(crate) fn numbering_system_selection(
        &self,
    ) -> Option<&[crate::number_format::NumberingSystemOption]> {
        match &self.domains {
            NumberLocaleDomains::Complete => None,
            NumberLocaleDomains::Projected {
                numbering_systems, ..
            } => numbering_systems.as_deref(),
        }
    }
    pub(crate) fn currency_codes(&self) -> Option<&[CurrencyCode]> {
        match &self.domains {
            NumberLocaleDomains::Complete => None,
            NumberLocaleDomains::Projected { currency_codes, .. } => currency_codes.as_deref(),
        }
    }
    pub(crate) fn default_numbering_index(&self, locale: &str) -> Option<u8> {
        match &self.domains {
            NumberLocaleDomains::Complete => self
                .profile(locale)
                .map(|profile| profile.default_numbering),
            NumberLocaleDomains::Projected { defaults, .. } => defaults
                .binary_search_by(|(name, _)| name.as_ref().cmp(locale))
                .ok()
                .map(|index| defaults[index].1),
        }
    }

    /// Only the actual image's exact rederivation admission can supply this
    /// catalogue. Hidden rows remain physical data, not public availability.
    pub(crate) fn from_projected_bytes(
        catalogue: &crate::number_image::projection::NumberCatalogue<'_>,
        locale: &crate::LocaleDataImage,
    ) -> Result<Self, InvalidNumberProfile> {
        let mut profiles = read::decode_projected(catalogue, locale)?;
        if profiles.locales.as_ref() != catalogue.physical_locales() {
            return Err(InvalidNumberProfile {
                table: NumberProfileTable::Locales,
                index: 0,
                reason: NumberProfileError::Cardinality,
            });
        }
        profiles.domains = catalogue.domains();
        Ok(profiles)
    }

    pub(crate) fn required_relative_time_locales(&self) -> Option<&[Box<str>]> {
        match &self.domains {
            NumberLocaleDomains::Complete => None,
            NumberLocaleDomains::Projected { relative_time, .. } => Some(relative_time),
        }
    }

    pub(crate) fn required_duration_locales(&self) -> Option<&[Box<str>]> {
        match &self.domains {
            NumberLocaleDomains::Complete => None,
            NumberLocaleDomains::Projected { duration, .. } => Some(duration),
        }
    }
}

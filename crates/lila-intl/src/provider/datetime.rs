//! Checked CLDR profiles, exact calendar fields and one parts renderer.

use crate::datetime::*;
use crate::{CanonicalLocaleId, IntlDataDigest, LocaleId};
use icu_provider_blob::BlobDataProvider;
use std::sync::Arc;

use super::named_time_zones::NamedTimeZones;
use super::{KeywordAliasData, LocaleCanonicalizationData};

mod calendar;
mod identity;
mod locale;
mod names;
mod pattern;
mod plan;
mod profile;
mod ranges;
mod raw;
mod render;
#[cfg(test)]
mod tests;
mod validation;
mod zones;

use profile::Profile;

pub(super) const PROVIDER_DATA_SHA256: [u8; 32] = identity::PROVIDER_DATA_SHA256;
const PLAN_MAGIC: &[u8; 8] = b"LILADTF2";

pub(crate) struct DateTimeProvider {
    profile: Profile,
    calendars: calendar::CalendarKernels,
    named_time_zones: Arc<NamedTimeZones>,
    keyword_aliases: Arc<KeywordAliasData>,
    image_digest: IntlDataDigest,
    locale_digest: IntlDataDigest,
    named_time_zone_digest: IntlDataDigest,
}

impl DateTimeProvider {
    pub(crate) fn from_image_data(
        source: &str,
        data: &BlobDataProvider,
        canonicalizer: &LocaleCanonicalizationData,
        keyword_aliases: Arc<KeywordAliasData>,
        named_time_zones: Arc<NamedTimeZones>,
        image_digest: IntlDataDigest,
        locale_digest: IntlDataDigest,
        named_time_zone_digest: IntlDataDigest,
    ) -> Result<Self, DateTimeFormatError> {
        let profile = Profile::from_json(source)?;
        Self::from_profile(
            profile,
            data,
            canonicalizer,
            keyword_aliases,
            named_time_zones,
            image_digest,
            locale_digest,
            named_time_zone_digest,
        )
    }

    pub(crate) fn from_projection(
        catalogue: &crate::datetime_image::projection::DateTimeCatalogue<'_>,
        data: &BlobDataProvider,
        canonicalizer: &LocaleCanonicalizationData,
        keyword_aliases: Arc<KeywordAliasData>,
        named_time_zones: Arc<NamedTimeZones>,
        image_digest: IntlDataDigest,
        locale_digest: IntlDataDigest,
        named_time_zone_digest: IntlDataDigest,
    ) -> Result<Self, DateTimeFormatError> {
        Self::from_profile(
            Profile::from_projection(catalogue)?,
            data,
            canonicalizer,
            keyword_aliases,
            named_time_zones,
            image_digest,
            locale_digest,
            named_time_zone_digest,
        )
    }

    /// Whole-source validation precedes selection; the image producer consumes
    /// this actual catalogue instead of treating arbitrary JSON rows as proof.
    pub(crate) fn projection_source_locales(
        source: &str,
        canonicalizer: &LocaleCanonicalizationData,
    ) -> Result<Vec<CanonicalLocaleId>, DateTimeFormatError> {
        let profile = Profile::from_json(source)?;
        Self::validate_locales(&profile, canonicalizer)?;
        Ok(profile
            .locales
            .into_iter()
            .map(|locale| locale.identifier)
            .collect())
    }

    fn validate_locales(
        profile: &Profile,
        canonicalizer: &LocaleCanonicalizationData,
    ) -> Result<(), DateTimeFormatError> {
        for locale in &profile.locales {
            let identifier = LocaleId::parse(locale.identifier.as_str())
                .map_err(|error| DateTimeFormatError::InvalidProfile(error.to_string()))?;
            let canonical = canonicalizer
                .canonicalize(&identifier)
                .map_err(|error| DateTimeFormatError::InvalidProfile(error.to_string()))?;
            if canonical != locale.identifier {
                return Err(DateTimeFormatError::InvalidProfile(
                    "noncanonical native DateTime locale".into(),
                ));
            }
        }
        Ok(())
    }

    fn from_profile(
        profile: Profile,
        data: &BlobDataProvider,
        canonicalizer: &LocaleCanonicalizationData,
        keyword_aliases: Arc<KeywordAliasData>,
        named_time_zones: Arc<NamedTimeZones>,
        image_digest: IntlDataDigest,
        locale_digest: IntlDataDigest,
        named_time_zone_digest: IntlDataDigest,
    ) -> Result<Self, DateTimeFormatError> {
        Self::validate_locales(&profile, canonicalizer)?;
        Ok(Self {
            profile,
            calendars: calendar::CalendarKernels::from_data(data)?,
            named_time_zones,
            keyword_aliases,
            image_digest,
            locale_digest,
            named_time_zone_digest,
        })
    }

    pub(crate) fn resolve_locale(
        &self,
        request: DateTimeLocaleRequest,
    ) -> Result<DateTimeLocaleResult, DateTimeFormatError> {
        locale::resolve(&self.profile, &self.keyword_aliases, request)
    }
    /// Localized service availability comes from actual selected name/pattern
    /// associations; global calculation support is a separate retained owner.
    pub(crate) fn available_calendars(&self) -> Result<Vec<DateTimeCalendar>, DateTimeFormatError> {
        let mut calendars = Vec::with_capacity(DateTimeCalendar::ALL.len());
        for expected in DateTimeCalendar::ALL.iter().copied() {
            if self
                .profile
                .locales
                .iter()
                .any(|locale| locale.supports_calendar(expected))
            {
                calendars.push(expected);
            }
        }
        Ok(calendars)
    }
    pub(crate) fn available_calendar_kernels(&self) -> Vec<DateTimeCalendar> {
        self.calendars.available_calendars()
    }
    pub(crate) fn available_numbering_system_kernels(&self) -> Vec<Box<str>> {
        self.profile
            .digits
            .keys()
            .map(|name| name.clone().into_boxed_str())
            .collect()
    }
    pub(crate) fn supported_locales(
        &self,
        request: DateTimeSupportedLocalesRequest,
    ) -> Result<DateTimeSupportedLocalesResult, DateTimeFormatError> {
        Ok(locale::supported(&self.profile, request))
    }
    pub(crate) fn select_plan(
        &self,
        request: DateTimePlanRequest,
    ) -> Result<DateTimePlanResult, DateTimeFormatError> {
        self.require_plan_time_zone(&request.time_zone)?;
        let selected = plan::select(&self.profile, &request)?;
        let recipe = request.encode().map_err(plan_wire_error)?;
        let mut bytes = Vec::with_capacity(104 + recipe.len());
        bytes.extend_from_slice(PLAN_MAGIC);
        bytes.extend_from_slice(self.image_digest.as_bytes());
        bytes.extend_from_slice(self.locale_digest.as_bytes());
        bytes.extend_from_slice(self.named_time_zone_digest.as_bytes());
        bytes.extend_from_slice(&recipe);
        Ok(DateTimePlanResult {
            plan: EncodedDateTimePlan::from_bytes(bytes),
            locale: request.locale,
            time_zone: request.time_zone,
            components: selected.legacy_components(),
            styles: match request.selection {
                DateTimeStyleSelection::Components(_) => None,
                DateTimeStyleSelection::Styles(styles) => Some(styles),
            },
            available_formats: selected.available_formats(),
        })
    }
    pub(crate) fn format_parts(
        &self,
        request: DateTimeFormatRequest,
    ) -> Result<DateTimeParts, DateTimeFormatError> {
        let recipe = self.decode_plan(&request.plan)?;
        let selected = plan::select(&self.profile, &recipe)?;
        render::format(
            &self.profile,
            &selected,
            request.input,
            &self.named_time_zones,
            &self.calendars,
        )
    }
    pub(crate) fn format_range_parts(
        &self,
        request: DateTimeRangeRequest,
    ) -> Result<DateTimeRangeParts, DateTimeFormatError> {
        let recipe = self.decode_plan(&request.plan)?;
        if request.start.kind() != request.end.kind() {
            return Err(DateTimeFormatError::InputKindMismatch);
        }
        let selected = plan::select(&self.profile, &recipe)?;
        ranges::format(
            &self.profile,
            &selected,
            request.start,
            request.end,
            &self.named_time_zones,
            &self.calendars,
        )
    }
    fn decode_plan(
        &self,
        plan: &EncodedDateTimePlan,
    ) -> Result<DateTimePlanRequest, DateTimeFormatError> {
        let bytes = plan.as_bytes();
        if bytes.get(..8) != Some(PLAN_MAGIC.as_slice())
            || bytes.get(8..40) != Some(self.image_digest.as_bytes().as_slice())
            || bytes.get(40..72) != Some(self.locale_digest.as_bytes().as_slice())
            || bytes.get(72..104) != Some(self.named_time_zone_digest.as_bytes().as_slice())
        {
            return Err(DateTimeFormatError::InvalidPlan(
                "schema or provider identity mismatch",
            ));
        }
        let request = DateTimePlanRequest::decode(&bytes[104..]).map_err(plan_wire_error)?;
        self.require_plan_time_zone(&request.time_zone)?;
        Ok(request)
    }

    pub(crate) fn uses_named_time_zones(&self, zones: &Arc<NamedTimeZones>) -> bool {
        Arc::ptr_eq(&self.named_time_zones, zones)
    }
    pub(crate) fn named_time_zone_selection(&self) -> Option<&[crate::TimeZoneId]> {
        self.named_time_zones.named_time_zone_selection()
    }
    fn require_plan_time_zone(
        &self,
        selection: &crate::TimeZoneSelection,
    ) -> Result<(), DateTimeFormatError> {
        if let crate::TimeZoneSelection::Named(name) = selection {
            let identity = self
                .named_time_zones
                .lookup(name)
                .map_err(|_| DateTimeFormatError::InvalidPlan("unknown named time zone"))?;
            self.named_time_zones
                .require_available(&identity)
                .map_err(|error| match error {
                    crate::NamedTimeZoneDataError::UnavailableService(service) => {
                        DateTimeFormatError::UnavailableService(service)
                    }
                    crate::NamedTimeZoneDataError::UnavailableIdentifier(name) => {
                        DateTimeFormatError::UnavailableTimeZone(name)
                    }
                    crate::NamedTimeZoneDataError::UnknownIdentifier(_) => {
                        DateTimeFormatError::InvalidPlan("unknown named time zone")
                    }
                    crate::NamedTimeZoneDataError::InvalidProviderData(error) => {
                        DateTimeFormatError::InvalidProfile(error.to_string())
                    }
                })?;
        }
        Ok(())
    }
}

fn plan_wire_error(error: crate::DateTimeWireError) -> DateTimeFormatError {
    match error {
        crate::DateTimeWireError::Domain(error) => error,
        crate::DateTimeWireError::Malformed(reason)
        | crate::DateTimeWireError::Resource(reason) => DateTimeFormatError::InvalidPlan(reason),
    }
}

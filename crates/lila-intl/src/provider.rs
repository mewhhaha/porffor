use core::fmt;

use icu_locale::{LocaleCanonicalizer, LocaleExpander};
use sha2::{Digest as _, Sha256};

use crate::number_format::{
    embedded_number_profiles, filter_number_locales, resolve_number_locale, InvalidNumberProfile,
    NumberLocaleRequest, NumberProfiles, NumberSupportedLocalesRequest, PartitionLimits,
    RangeNumberPartition, ResolvedNumberLocale, ScalarNumberPartition, NUMBER_FORMAT_DATA_SHA256,
};
use crate::number_operation::{format_number_parts_operation, format_number_range_parts_operation};
use crate::{
    FormatNumberParts, FormatNumberRangeParts, NumberFormatOperationError, NumberFormatRequest,
    NumberRangeFormatRequest, NumberSupportedLocalesResult, ResolveNumberLocale,
    SupportedNumberLocales,
};

mod datetime;
mod keyword_aliases;
mod language_domain;
#[cfg(test)]
mod likely_subtags_tests;
mod named_time_zones;
mod time_zone_names;
mod time_zone_snapshot;
use language_domain::{LikelySubtags, ParsedLocale, ReservedLanguageAliasRules};
use named_time_zones::NamedTimeZones;
use time_zone_names::TimeZoneNames;
use time_zone_snapshot::TimeZoneNameInput;

use crate::{
    CanonicalLocaleId, CanonicalizeLocale, DateTimeFormatError, DateTimeFormatRequest,
    DateTimeLocaleRequest, DateTimeLocaleResult, DateTimeParts, DateTimePlanRequest,
    DateTimePlanResult, DateTimeRangeParts, DateTimeRangeRequest, DateTimeSupportedLocalesRequest,
    DateTimeSupportedLocalesResult, EmptyIntlProfile, FormatDateTimeParts,
    FormatDateTimeRangeParts, IntlDataDigest, IntlDataIdentity, IntlDataPlacement,
    IntlOperationProvider, IntlProfilePlan, IntlProvider, IntlService, IntlServiceSet,
    InvalidCanonicalLocaleId, InvalidTimeZoneData, LocaleTransformError, LocaleTransformRequest,
    LocaleTransformResult, LookupNamedTimeZone, LookupNamedTimeZoneRequest,
    LookupNamedTimeZoneResult, MaximizeLocale, MinimizeLocale, ResolveDateTimeLocale,
    ResolveTimeZone, ResolveTimeZoneRequest, ResolvedTimeZoneSnapshot, SelectDateTimeFormat,
    SupportedDateTimeLocales, TimeZoneResolveError, TimeZoneSelection, UnknownTimeZone,
};

/// Composite SHA-256 of exact locale, IANA transition/catalogue and CLDR name
/// manifests. Recompute from their generated digests so no independently stored
/// outer digest can remain stale when a component is regenerated.
fn embedded_intl_data_digest() -> IntlDataDigest {
    let mut digest = Sha256::new();
    digest.update(b"lila-intl-provider-v5\0");
    digest.update(keyword_aliases::PROVIDER_DATA_SHA256);
    digest.update(named_time_zones::PROVIDER_DATA_SHA256);
    digest.update(time_zone_names::PROVIDER_DATA_SHA256);
    digest.update(datetime::PROVIDER_DATA_SHA256);
    digest.update(NUMBER_FORMAT_DATA_SHA256);
    IntlDataDigest::from_sha256(digest.finalize().into())
}

/// Pure locale transforms, time-zone snapshots, and date/time and number partitions.
/// All data is pinned in the Rust host and is External to the Wasm artifact.
pub struct EmbeddedIntlProvider {
    identity: IntlDataIdentity,
    canonicalizer: LocaleCanonicalizer,
    expander: LocaleExpander,
    reserved_language_rules: ReservedLanguageAliasRules,
    named_time_zones: NamedTimeZones,
    time_zone_names: TimeZoneNames,
    date_time: datetime::DateTimeProvider,
    numbers: &'static NumberProfiles,
}

impl fmt::Debug for EmbeddedIntlProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmbeddedIntlProvider")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

impl EmbeddedIntlProvider {
    pub fn new() -> Result<Self, EmbeddedIntlProviderSetupError> {
        let identity = embedded_intl_data_identity()?;
        Ok(Self {
            identity,
            canonicalizer: LocaleCanonicalizer::new_extended(),
            expander: LocaleExpander::new_extended(),
            reserved_language_rules: ReservedLanguageAliasRules::from_pinned_data()
                .map_err(EmbeddedIntlProviderSetupError::ReservedLanguageData)?,
            named_time_zones: NamedTimeZones::from_pinned_data()
                .map_err(EmbeddedIntlProviderSetupError::TimeZoneData)?,
            time_zone_names: TimeZoneNames::from_pinned_data()
                .map_err(EmbeddedIntlProviderSetupError::TimeZoneNameData)?,
            date_time: datetime::DateTimeProvider::from_pinned_data()
                .map_err(EmbeddedIntlProviderSetupError::DateTimeData)?,
            numbers: embedded_number_profiles()
                .map_err(EmbeddedIntlProviderSetupError::NumberData)?,
        })
    }
}

/// Identity expected by artifacts that use the host-embedded Intl provider.
///
/// Kept separate from [`EmbeddedIntlProvider::new`] so an AOT emitter can
/// carry the exact provider identity without constructing the ICU canonicalizer
/// it will never execute.
pub fn embedded_intl_data_identity() -> Result<IntlDataIdentity, EmbeddedIntlProviderSetupError> {
    let services = IntlServiceSet::EMPTY
        .with(IntlService::Locale)
        .with(IntlService::DateTimeFormat)
        .with(IntlService::NumberFormat);
    let profile = IntlProfilePlan::minimal(services)
        .map_err(EmbeddedIntlProviderSetupError::EmptyProfile)?
        .with_operation::<LookupNamedTimeZone>()
        .with_operation::<ResolveTimeZone>();
    let default_locale = CanonicalLocaleId::from_data("en-US")
        .map_err(EmbeddedIntlProviderSetupError::InvalidDefaultLocale)?;
    Ok(IntlDataIdentity::new(
        profile,
        default_locale,
        IntlDataPlacement::External,
        embedded_intl_data_digest(),
    ))
}

impl IntlProvider for EmbeddedIntlProvider {
    fn identity(&self) -> &IntlDataIdentity {
        &self.identity
    }
}

impl IntlOperationProvider<CanonicalizeLocale> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: LocaleTransformRequest,
    ) -> Result<LocaleTransformResult, LocaleTransformError> {
        let input = request.into_locale();
        let parsed = ParsedLocale::parse(&input)?;
        let canonical = parsed.canonicalize(&self.canonicalizer, &self.reserved_language_rules)?;
        Ok(LocaleTransformResult::new(canonical))
    }
}

impl IntlOperationProvider<MaximizeLocale> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: LocaleTransformRequest,
    ) -> Result<LocaleTransformResult, LocaleTransformError> {
        let locale = ParsedLocale::parse(&request.into_locale())?.apply_likely_subtags(
            LikelySubtags::Maximize,
            &self.expander,
            &self.canonicalizer,
            &self.reserved_language_rules,
        )?;
        Ok(LocaleTransformResult::new(locale))
    }
}

impl IntlOperationProvider<MinimizeLocale> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: LocaleTransformRequest,
    ) -> Result<LocaleTransformResult, LocaleTransformError> {
        let locale = ParsedLocale::parse(&request.into_locale())?.apply_likely_subtags(
            LikelySubtags::Minimize,
            &self.expander,
            &self.canonicalizer,
            &self.reserved_language_rules,
        )?;
        Ok(LocaleTransformResult::new(locale))
    }
}

impl IntlOperationProvider<LookupNamedTimeZone> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: LookupNamedTimeZoneRequest,
    ) -> Result<LookupNamedTimeZoneResult, UnknownTimeZone> {
        self.named_time_zones
            .lookup(request.identifier())
            .map(LookupNamedTimeZoneResult::new)
    }
}

impl IntlOperationProvider<ResolveTimeZone> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: ResolveTimeZoneRequest,
    ) -> Result<ResolvedTimeZoneSnapshot, TimeZoneResolveError> {
        let identity;
        let (offset_seconds, input) = match request.selection() {
            TimeZoneSelection::Named(identifier) => {
                identity = self.named_time_zones.lookup(identifier).map_err(|_| {
                    TimeZoneResolveError::InvalidNamedIdentifier(identifier.clone())
                })?;
                let transition = self
                    .named_time_zones
                    .transition(&identity, request.epoch())?;
                (
                    transition.offset_seconds(),
                    TimeZoneNameInput::Named {
                        identity: &identity,
                        epoch: request.epoch(),
                        transition,
                    },
                )
            }
            TimeZoneSelection::FixedOffset(offset) => {
                (offset.seconds(), TimeZoneNameInput::FixedOffset(*offset))
            }
        };
        let display_name = request
            .name_style()
            .map(|style| self.time_zone_names.format(input, style, request.locale()))
            .transpose()?;
        ResolvedTimeZoneSnapshot::from_data(offset_seconds, display_name).map_err(Into::into)
    }
}

impl IntlOperationProvider<ResolveDateTimeLocale> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: DateTimeLocaleRequest,
    ) -> Result<DateTimeLocaleResult, DateTimeFormatError> {
        self.date_time.resolve_locale(request)
    }
}

impl IntlOperationProvider<SupportedDateTimeLocales> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: DateTimeSupportedLocalesRequest,
    ) -> Result<DateTimeSupportedLocalesResult, DateTimeFormatError> {
        self.date_time.supported_locales(request)
    }
}

impl IntlOperationProvider<SelectDateTimeFormat> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: DateTimePlanRequest,
    ) -> Result<DateTimePlanResult, DateTimeFormatError> {
        self.date_time.select_plan(request)
    }
}

impl IntlOperationProvider<FormatDateTimeParts> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: DateTimeFormatRequest,
    ) -> Result<DateTimeParts, DateTimeFormatError> {
        self.date_time.format_parts(request, &self.named_time_zones)
    }
}

impl IntlOperationProvider<FormatDateTimeRangeParts> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: DateTimeRangeRequest,
    ) -> Result<DateTimeRangeParts, DateTimeFormatError> {
        self.date_time
            .format_range_parts(request, &self.named_time_zones)
    }
}

impl IntlOperationProvider<ResolveNumberLocale> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: NumberLocaleRequest,
    ) -> Result<ResolvedNumberLocale, NumberFormatOperationError> {
        Ok(resolve_number_locale(&request, self.numbers)?)
    }
}

impl IntlOperationProvider<SupportedNumberLocales> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: NumberSupportedLocalesRequest,
    ) -> Result<NumberSupportedLocalesResult, NumberFormatOperationError> {
        Ok(NumberSupportedLocalesResult {
            locales: filter_number_locales(&request, self.numbers)?,
        })
    }
}

impl IntlOperationProvider<FormatNumberParts> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: NumberFormatRequest,
    ) -> Result<ScalarNumberPartition, NumberFormatOperationError> {
        format_number_parts_operation(request, self.numbers, &PartitionLimits::HOST_ABI)
    }
}

impl IntlOperationProvider<FormatNumberRangeParts> for EmbeddedIntlProvider {
    fn execute(
        &self,
        request: NumberRangeFormatRequest,
    ) -> Result<RangeNumberPartition, NumberFormatOperationError> {
        format_number_range_parts_operation(request, self.numbers, &PartitionLimits::HOST_ABI)
    }
}

#[derive(Debug)]
pub enum EmbeddedIntlProviderSetupError {
    EmptyProfile(EmptyIntlProfile),
    InvalidDefaultLocale(InvalidCanonicalLocaleId),
    ReservedLanguageData(&'static str),
    TimeZoneData(InvalidTimeZoneData),
    TimeZoneNameData(InvalidTimeZoneData),
    DateTimeData(DateTimeFormatError),
    NumberData(InvalidNumberProfile),
}

impl fmt::Display for EmbeddedIntlProviderSetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProfile(error) => error.fmt(f),
            Self::InvalidDefaultLocale(error) => error.fmt(f),
            Self::TimeZoneData(error) => write!(f, "pinned IANA data is incompatible: {error}"),
            Self::TimeZoneNameData(error) => {
                write!(f, "pinned time-zone name data is incompatible: {error}")
            }
            Self::DateTimeData(error) => {
                write!(f, "pinned date/time data is incompatible: {error}")
            }
            Self::NumberData(error) => {
                write!(f, "pinned number data is incompatible: {error}")
            }
            Self::ReservedLanguageData(reason) => write!(
                f,
                "pinned Intl reserved-language data is incompatible: {reason}"
            ),
        }
    }
}

impl std::error::Error for EmbeddedIntlProviderSetupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EmptyProfile(error) => Some(error),
            Self::InvalidDefaultLocale(error) => Some(error),
            Self::ReservedLanguageData(_) => None,
            Self::TimeZoneData(error) | Self::TimeZoneNameData(error) => Some(error),
            Self::DateTimeData(error) => Some(error),
            Self::NumberData(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntlDataCapability, IntlKernel, LocaleId, LocaleTransformRequest};

    #[test]
    fn embedded_locale_provider_resolves_cldr_aliases() {
        let provider = EmbeddedIntlProvider::new().expect("embedded profile is valid");
        let identity = provider.identity().clone();
        assert_eq!(
            identity,
            embedded_intl_data_identity().expect("embedded profile identity is valid")
        );
        assert_eq!(identity.placement(), IntlDataPlacement::External);
        assert_eq!(identity.digest(), embedded_intl_data_digest());
        assert!(identity
            .profile()
            .capabilities()
            .contains(IntlDataCapability::LocaleAliases));

        let kernel = IntlKernel::new(identity, provider).expect("provider identity matches");
        let result = kernel
            .operation::<CanonicalizeLocale>()
            .expect("locale capability is present")
            .execute(LocaleTransformRequest::new(
                LocaleId::parse("iw-IL").expect("structurally valid locale"),
            ))
            .expect("pinned ICU4X data contains the alias");

        assert_eq!(result.locale().as_str(), "he-IL");
    }
}

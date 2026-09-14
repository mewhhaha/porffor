use core::fmt;

use icu_locale::LocaleCanonicalizer;

mod keyword_aliases;
mod language_domain;
use language_domain::{ParsedLocale, ReservedLanguageAliasRules};

use crate::{
    CanonicalLocaleId, CanonicalizeLocale, CanonicalizeLocaleError, CanonicalizeLocaleRequest,
    CanonicalizeLocaleResult, EmptyIntlProfile, IntlDataDigest, IntlDataIdentity,
    IntlDataPlacement, IntlOperationProvider, IntlProfilePlan, IntlProvider, IntlService,
    IntlServiceSet, InvalidCanonicalLocaleId,
};

/// Composite SHA-256 of the pinned ICU locale archive, CLDR BCP47 source
/// manifest and generated alias rows. The generator owns this exact recipe;
/// these tables are embedded in the host, not in emitted Wasm artifacts.
pub const EMBEDDED_LOCALE_DATA_SHA256: IntlDataDigest =
    IntlDataDigest::from_sha256(keyword_aliases::PROVIDER_DATA_SHA256);

/// Pure locale canonicalization backed by ICU4X's compiled CLDR 47 data.
///
/// The data is compiled into the Rust host, so its identity is `External` from
/// the emitted Wasm artifact's point of view. This provider deliberately binds
/// only the Locale service; time-zone canonicalization remains unbound.
#[derive(Debug)]
pub struct EmbeddedLocaleProvider {
    identity: IntlDataIdentity,
    canonicalizer: LocaleCanonicalizer,
    reserved_language_rules: ReservedLanguageAliasRules,
}

impl EmbeddedLocaleProvider {
    pub fn new() -> Result<Self, EmbeddedLocaleProviderSetupError> {
        let identity = embedded_locale_data_identity()?;
        Ok(Self {
            identity,
            canonicalizer: LocaleCanonicalizer::new_extended(),
            reserved_language_rules: ReservedLanguageAliasRules::from_pinned_data()
                .map_err(EmbeddedLocaleProviderSetupError::ReservedLanguageData)?,
        })
    }
}

/// Identity expected by artifacts that use the host-embedded Locale provider.
///
/// Kept separate from [`EmbeddedLocaleProvider::new`] so an AOT emitter can
/// carry the exact provider identity without constructing the ICU canonicalizer
/// it will never execute.
pub fn embedded_locale_data_identity() -> Result<IntlDataIdentity, EmbeddedLocaleProviderSetupError>
{
    let services = IntlServiceSet::EMPTY.with(IntlService::Locale);
    let profile = IntlProfilePlan::minimal(services)
        .map_err(EmbeddedLocaleProviderSetupError::EmptyProfile)?;
    let default_locale = CanonicalLocaleId::from_data("en-US")
        .map_err(EmbeddedLocaleProviderSetupError::InvalidDefaultLocale)?;
    Ok(IntlDataIdentity::new(
        profile,
        default_locale,
        IntlDataPlacement::External,
        EMBEDDED_LOCALE_DATA_SHA256,
    ))
}

impl IntlProvider for EmbeddedLocaleProvider {
    fn identity(&self) -> &IntlDataIdentity {
        &self.identity
    }
}

impl IntlOperationProvider<CanonicalizeLocale> for EmbeddedLocaleProvider {
    fn execute(
        &self,
        request: CanonicalizeLocaleRequest,
    ) -> Result<CanonicalizeLocaleResult, CanonicalizeLocaleError> {
        let input = request.into_locale();
        let parsed = ParsedLocale::parse(&input)?;
        let canonical = parsed.canonicalize(&self.canonicalizer, &self.reserved_language_rules)?;
        Ok(CanonicalizeLocaleResult::new(canonical))
    }
}

#[derive(Debug)]
pub enum EmbeddedLocaleProviderSetupError {
    EmptyProfile(EmptyIntlProfile),
    InvalidDefaultLocale(InvalidCanonicalLocaleId),
    ReservedLanguageData(&'static str),
}

impl fmt::Display for EmbeddedLocaleProviderSetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProfile(error) => error.fmt(f),
            Self::InvalidDefaultLocale(error) => error.fmt(f),
            Self::ReservedLanguageData(reason) => write!(
                f,
                "pinned Intl reserved-language data is incompatible: {reason}"
            ),
        }
    }
}

impl std::error::Error for EmbeddedLocaleProviderSetupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EmptyProfile(error) => Some(error),
            Self::InvalidDefaultLocale(error) => Some(error),
            Self::ReservedLanguageData(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CanonicalizeLocaleRequest, IntlDataCapability, IntlKernel, LocaleId};

    #[test]
    fn embedded_locale_provider_resolves_cldr_aliases() {
        let provider = EmbeddedLocaleProvider::new().expect("embedded profile is valid");
        let identity = provider.identity().clone();
        assert_eq!(
            identity,
            embedded_locale_data_identity().expect("embedded profile identity is valid")
        );
        assert_eq!(identity.placement(), IntlDataPlacement::External);
        assert_eq!(identity.digest(), EMBEDDED_LOCALE_DATA_SHA256);
        assert!(identity
            .profile()
            .capabilities()
            .contains(IntlDataCapability::LocaleAliases));

        let kernel = IntlKernel::new(identity, provider).expect("provider identity matches");
        let result = kernel
            .operation::<CanonicalizeLocale>()
            .expect("locale capability is present")
            .execute(CanonicalizeLocaleRequest::new(
                LocaleId::parse("iw-IL").expect("structurally valid locale"),
            ))
            .expect("pinned ICU4X data contains the alias");

        assert_eq!(result.locale().as_str(), "he-IL");
    }
}

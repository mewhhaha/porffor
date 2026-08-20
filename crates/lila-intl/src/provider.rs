use core::fmt;

use icu_locale::{Locale, LocaleCanonicalizer};

use crate::{
    CanonicalLocaleId, CanonicalizeLocale, CanonicalizeLocaleError, CanonicalizeLocaleRequest,
    CanonicalizeLocaleResult, EmptyIntlProfile, IntlDataDigest, IntlDataIdentity,
    IntlDataPlacement, IntlOperationProvider, IntlProfilePlan, IntlProvider, IntlService,
    IntlServiceSet, InvalidCanonicalLocaleId, UnsupportedLocale,
};

/// SHA-256 of the exactly pinned `icu_locale_data-2.0.0.crate` archive.
///
/// This identifies the host-embedded locale tables used by this provider. It
/// is not a claim that those bytes are embedded in an emitted Wasm artifact.
pub const EMBEDDED_LOCALE_DATA_SHA256: IntlDataDigest = IntlDataDigest::from_sha256([
    0x4f, 0xde, 0xf0, 0xc1, 0x24, 0x74, 0x9d, 0x06, 0xa7, 0x43, 0xc6, 0x9e, 0x93, 0x83, 0x50, 0x81,
    0x65, 0x54, 0xeb, 0x63, 0xac, 0x97, 0x91, 0x66, 0x59, 0x0e, 0x2b, 0x4e, 0xe4, 0x25, 0x27, 0x65,
]);

/// Pure locale canonicalization backed by ICU4X's compiled CLDR 47 data.
///
/// The data is compiled into the Rust host, so its identity is `External` from
/// the emitted Wasm artifact's point of view. This provider deliberately binds
/// only the Locale service; time-zone canonicalization remains unbound.
#[derive(Debug)]
pub struct EmbeddedLocaleProvider {
    identity: IntlDataIdentity,
    canonicalizer: LocaleCanonicalizer,
}

impl EmbeddedLocaleProvider {
    pub fn new() -> Result<Self, EmbeddedLocaleProviderSetupError> {
        let identity = embedded_locale_data_identity()?;
        Ok(Self {
            identity,
            canonicalizer: LocaleCanonicalizer::new_extended(),
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
        let mut locale = input
            .as_str()
            .parse::<Locale>()
            .map_err(|_| UnsupportedLocale::new(input.clone()))?;
        self.canonicalizer.canonicalize(&mut locale);
        let canonical = CanonicalLocaleId::from_data(locale.to_string().into_boxed_str())?;
        Ok(CanonicalizeLocaleResult::new(canonical))
    }
}

#[derive(Debug)]
pub enum EmbeddedLocaleProviderSetupError {
    EmptyProfile(EmptyIntlProfile),
    InvalidDefaultLocale(InvalidCanonicalLocaleId),
}

impl fmt::Display for EmbeddedLocaleProviderSetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProfile(error) => error.fmt(f),
            Self::InvalidDefaultLocale(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for EmbeddedLocaleProviderSetupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EmptyProfile(error) => Some(error),
            Self::InvalidDefaultLocale(error) => Some(error),
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

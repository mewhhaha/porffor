//! Deterministic ECMA-402 data and protocol domains.
//!
//! This crate owns the closed vocabulary shared by the data generator, Wasm
//! emitter and runtime provider. Its host-embedded kernel handles pinned locale
//! transforms and exact IANA time-zone snapshots without access to parser state,
//! JavaScript IR, Wasmtime, JavaScript objects or observable operations.

use core::{fmt, fmt::Write as _};

mod datetime;
mod datetime_protocol;
mod identifiers;
pub mod number_format;
mod number_operation;
mod number_protocol;
mod protocol;
mod provider;
mod time_zone;

pub use datetime::{
    DateTimeCalendar, DateTimeComponents, DateTimeDefaults, DateTimeExactInput,
    DateTimeFormatAvailability, DateTimeFormatError, DateTimeFormatMatcher, DateTimeFormatRequest,
    DateTimeFractionalDigits, DateTimeHourCycle, DateTimeHourCyclePreference, DateTimeInput,
    DateTimeIsoFields, DateTimeKeyword, DateTimeLocaleMatcher, DateTimeLocaleRequest,
    DateTimeLocaleResult, DateTimeMonthWidth, DateTimeNumericWidth, DateTimePart, DateTimePartKind,
    DateTimeParts, DateTimePlainInput, DateTimePlanRequest, DateTimePlanResult, DateTimeRangePart,
    DateTimeRangeParts, DateTimeRangeRequest, DateTimeRangeSource, DateTimeRequired, DateTimeStyle,
    DateTimeStyleSelection, DateTimeStyles, DateTimeSupportedLocalesRequest,
    DateTimeSupportedLocalesResult, DateTimeTextWidth, DateTimeValueKind, EncodedDateTimePlan,
};
pub use datetime_protocol::{
    DateTimeWireError, DATE_TIME_COMPONENT_COUNT, DATE_TIME_INPUT_BYTES,
    DATE_TIME_WIRE_HEADER_BYTES, DATE_TIME_WIRE_VERSION,
};

pub use number_operation::{
    NumberFormatOperationError, NumberFormatRequest, NumberRangeFormatRequest,
    NumberSupportedLocalesResult,
};
pub use number_protocol::{
    NumberConfigurationWord, NumberNumericKind, NumberPrecisionKind, NumberWireError,
    NUMBER_APPROXIMATELY_SIGN_CODE, NUMBER_CONFIGURATION_WORDS, NUMBER_WIRE_HEADER_BYTES,
    NUMBER_WIRE_VERSION,
};

pub use time_zone::{
    FixedTimeZoneOffset, InvalidTimeZoneData, InvalidTimeZoneRequest, LookupNamedTimeZoneRequest,
    LookupNamedTimeZoneResult, NamedTimeZoneIdentity, ResolveTimeZoneRequest,
    ResolvedTimeZoneSnapshot, TimeZoneEpochSeconds, TimeZoneKind, TimeZoneNameStyle,
    TimeZoneResolveError, TimeZoneSelection, LOOKUP_TIME_ZONE_HEADER_BYTES,
    LOOKUP_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET, LOOKUP_TIME_ZONE_PRIMARY_LENGTH_OFFSET,
    RESOLVE_TIME_ZONE_EPOCH_SECONDS_OFFSET, RESOLVE_TIME_ZONE_FIXED_SECONDS_OFFSET,
    RESOLVE_TIME_ZONE_HEADER_BYTES, RESOLVE_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET,
    RESOLVE_TIME_ZONE_KIND_OFFSET, RESOLVE_TIME_ZONE_LOCALE_LENGTH_OFFSET,
    RESOLVE_TIME_ZONE_NAME_STYLE_OFFSET, RESOLVE_TIME_ZONE_RESULT_HEADER_BYTES,
};

pub use identifiers::{
    CanonicalLocaleId, InvalidCanonicalLocaleId, InvalidLocaleId, InvalidTimeZoneId, LocaleId,
    TimeZoneId, MAX_TIME_ZONE_IDENTIFIER_BYTES,
};
pub use protocol::{
    CanonicalizeLocale, FormatDateTimeParts, FormatDateTimeRangeParts, FormatNumberParts,
    FormatNumberRangeParts, IntlHostCallOutcome, IntlHostOp, IntlHostReadSpan, IntlHostWriteSpan,
    IntlKernel, IntlOperation, IntlOperationHandle, IntlOperationProvider, IntlProvider,
    IntlProviderIdentityMismatch, LocaleTransformError, LocaleTransformRequest,
    LocaleTransformResult, LookupNamedTimeZone, MaximizeLocale, MinimizeLocale,
    MissingIntlCapabilities, ResolveDateTimeLocale, ResolveNumberLocale, ResolveTimeZone,
    SelectDateTimeFormat, SupportedDateTimeLocales, SupportedNumberLocales, UnknownTimeZone,
    UnsupportedLocale,
};
pub use provider::{
    embedded_intl_data_identity, EmbeddedIntlProvider, EmbeddedIntlProviderSetupError,
};

// The embedded provider crosses Wasmtime store and agent-thread boundaries.
// Keep that ownership contract checked where the ICU payload type is selected:
// `icu_provider/sync` uses `Arc` rather than `Rc` for owned data carts.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<IntlKernel<EmbeddedIntlProvider>>();
};

macro_rules! closed_string_domain {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($variant:ident => $wire_name:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $wire_name),+
                }
            }
        }
    };
}

pub const INTL_DATA_SCHEMA_VERSION: IntlDataSchemaVersion = IntlDataSchemaVersion(1);

/// Host-call ABI5 adds typed NumberFormat locale and partition operations.
/// Artifact identity includes this value so an incompatible host is rejected
/// before instantiation, independently of the pinned ICU/CLDR data identity.
pub const INTL_HOST_CALL_ABI_VERSION: u16 = 5;

/// Canonical Wasm custom section carrying the Intl provider identity expected
/// by a compiled artifact.
///
/// This section carries identity metadata, not the ICU/CLDR payload itself.
/// The current [`EmbeddedIntlProvider`] therefore records `External`
/// placement even though its Rust-side data is compiled into the host.
pub const INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION: &str = "lila.intl-data-identity.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntlDataSchemaVersion(u16);

impl IntlDataSchemaVersion {
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    pub fn decode(raw: u16) -> Result<Self, UnsupportedIntlDataSchema> {
        if raw == INTL_DATA_SCHEMA_VERSION.0 {
            Ok(INTL_DATA_SCHEMA_VERSION)
        } else {
            Err(UnsupportedIntlDataSchema { found: raw })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedIntlDataSchema {
    pub found: u16,
}

impl fmt::Display for UnsupportedIntlDataSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unsupported Intl data schema {}; expected {}",
            self.found,
            INTL_DATA_SCHEMA_VERSION.get()
        )
    }
}

impl std::error::Error for UnsupportedIntlDataSchema {}

closed_string_domain! {
    /// Intl service families selected by a data profile.
    pub enum IntlService {
        Locale => "Locale",
        Collator => "Collator",
        NumberFormat => "NumberFormat",
        DateTimeFormat => "DateTimeFormat",
        PluralRules => "PluralRules",
        RelativeTimeFormat => "RelativeTimeFormat",
        ListFormat => "ListFormat",
        DisplayNames => "DisplayNames",
        Segmenter => "Segmenter",
        DurationFormat => "DurationFormat",
    }
}

impl IntlService {
    #[must_use]
    pub const fn required_capabilities(self) -> IntlCapabilitySet {
        let common = IntlCapabilitySet::COMMON_LOCALE;
        match self {
            Self::Locale => common,
            Self::Collator => common.with(IntlDataCapability::Collation),
            Self::NumberFormat => common
                .with(IntlDataCapability::NumberingSystems)
                .with(IntlDataCapability::DecimalPatterns)
                .with(IntlDataCapability::PluralRules)
                .with(IntlDataCapability::UnitsAndCurrencies),
            Self::DateTimeFormat => common
                .with(IntlDataCapability::Calendars)
                .with(IntlDataCapability::NumberingSystems)
                .with(IntlDataCapability::DateTimePatterns)
                .with(IntlDataCapability::TimeZoneNames)
                .with(IntlDataCapability::TimeZoneTransitions),
            Self::PluralRules => common
                .with(IntlDataCapability::DecimalPatterns)
                .with(IntlDataCapability::PluralRules),
            Self::RelativeTimeFormat => common
                .with(IntlDataCapability::DecimalPatterns)
                .with(IntlDataCapability::PluralRules)
                .with(IntlDataCapability::RelativeTimePatterns),
            Self::ListFormat => common.with(IntlDataCapability::ListPatterns),
            Self::DisplayNames => common.with(IntlDataCapability::DisplayNames),
            Self::Segmenter => common.with(IntlDataCapability::Segmentation),
            Self::DurationFormat => common
                .with(IntlDataCapability::NumberingSystems)
                .with(IntlDataCapability::DecimalPatterns)
                .with(IntlDataCapability::ListPatterns)
                .with(IntlDataCapability::UnitsAndCurrencies),
        }
    }
}

const _: () = {
    let mut mask = 0u16;
    let mut i = 0;
    while i < IntlService::ALL.len() {
        mask |= 1u16 << IntlService::ALL[i] as u8;
        i += 1;
    }
    assert!(mask == (1u16 << IntlService::ALL.len()) - 1);
};

closed_string_domain! {
    /// Immutable data capabilities closed over selected services.
    pub enum IntlDataCapability {
        LocaleAliases => "locale-aliases",
        LikelySubtags => "likely-subtags",
        ParentLocales => "parent-locales",
        Calendars => "calendars",
        NumberingSystems => "numbering-systems",
        DecimalPatterns => "decimal-patterns",
        PluralRules => "plural-rules",
        Collation => "collation",
        DateTimePatterns => "date-time-patterns",
        TimeZoneNames => "time-zone-names",
        TimeZoneTransitions => "time-zone-transitions",
        RelativeTimePatterns => "relative-time-patterns",
        ListPatterns => "list-patterns",
        DisplayNames => "display-names",
        Segmentation => "segmentation",
        UnitsAndCurrencies => "units-and-currencies",
    }
}

const _: () = {
    let mut mask = 0u32;
    let mut i = 0;
    while i < IntlDataCapability::ALL.len() {
        mask |= 1u32 << IntlDataCapability::ALL[i] as u8;
        i += 1;
    }
    assert!(mask == (1u32 << IntlDataCapability::ALL.len()) - 1);
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntlServiceSet(u16);

impl IntlServiceSet {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self((1u16 << IntlService::ALL.len()) - 1);

    #[must_use]
    pub const fn with(self, service: IntlService) -> Self {
        Self(self.0 | (1u16 << service as u8))
    }

    #[must_use]
    pub const fn contains(self, service: IntlService) -> bool {
        self.0 & (1u16 << service as u8) != 0
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = IntlService> {
        IntlService::ALL
            .iter()
            .copied()
            .filter(move |service| self.contains(*service))
    }
}

impl FromIterator<IntlService> for IntlServiceSet {
    fn from_iter<T: IntoIterator<Item = IntlService>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Self::EMPTY, |services, service| services.with(service))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntlCapabilitySet(u32);

impl IntlCapabilitySet {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self((1u32 << IntlDataCapability::ALL.len()) - 1);
    pub const COMMON_LOCALE: Self = Self::EMPTY
        .with(IntlDataCapability::LocaleAliases)
        .with(IntlDataCapability::LikelySubtags)
        .with(IntlDataCapability::ParentLocales);

    #[must_use]
    pub const fn with(self, capability: IntlDataCapability) -> Self {
        Self(self.0 | (1u32 << capability as u8))
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, capability: IntlDataCapability) -> bool {
        self.0 & (1u32 << capability as u8) != 0
    }

    #[must_use]
    pub const fn contains_all(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }

    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = IntlDataCapability> {
        IntlDataCapability::ALL
            .iter()
            .copied()
            .filter(move |capability| self.contains(*capability))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CustomProfileId(Box<str>);

impl CustomProfileId {
    pub fn parse(raw: impl Into<Box<str>>) -> Result<Self, InvalidCustomProfileId> {
        let raw = raw.into();
        let bytes = raw.as_bytes();
        let valid = (1..=64).contains(&bytes.len())
            && bytes[0].is_ascii_alphanumeric()
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
        if valid {
            Ok(Self(raw))
        } else {
            Err(InvalidCustomProfileId { value: raw })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCustomProfileId {
    value: Box<str>,
}

impl fmt::Display for InvalidCustomProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid custom Intl profile id {:?}", self.value)
    }
}

impl std::error::Error for InvalidCustomProfileId {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntlDataProfile {
    Conformance,
    Minimal,
    Custom(CustomProfileId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntlDataPlacement {
    Embedded,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntlDataDigest([u8; 32]);

impl IntlDataDigest {
    #[must_use]
    pub const fn from_sha256(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntlDataVersions {
    pub icu4x: &'static str,
    pub cldr: &'static str,
    pub unicode: &'static str,
    pub icu_data_tag: &'static str,
    pub segmenter_lstm: &'static str,
    pub tzdb: &'static str,
}

impl IntlDataVersions {
    pub const PINNED: Self = Self {
        icu4x: "2.0",
        cldr: "47.0.0",
        unicode: "16.0.0",
        icu_data_tag: "icu4x/2025-05-01/77.x",
        segmenter_lstm: "v0.1.0",
        tzdb: "2026a",
    };
}

/// A selected profile with capability closure already computed.
///
/// Callers select services or sealed operations, never capability bits.
/// Advertising a complete DateTimeFormat service still requires its full data
/// closure; an individual operation can require a narrower set without claiming
/// that every other operation in that service is implemented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntlProfilePlan {
    profile: IntlDataProfile,
    services: IntlServiceSet,
    capabilities: IntlCapabilitySet,
}

impl IntlProfilePlan {
    #[must_use]
    pub const fn conformance() -> Self {
        Self {
            profile: IntlDataProfile::Conformance,
            services: IntlServiceSet::ALL,
            capabilities: IntlCapabilitySet::ALL,
        }
    }

    pub fn minimal(services: IntlServiceSet) -> Result<Self, EmptyIntlProfile> {
        Self::selected(IntlDataProfile::Minimal, services)
    }

    pub fn custom(id: CustomProfileId, services: IntlServiceSet) -> Result<Self, EmptyIntlProfile> {
        Self::selected(IntlDataProfile::Custom(id), services)
    }

    fn selected(
        profile: IntlDataProfile,
        services: IntlServiceSet,
    ) -> Result<Self, EmptyIntlProfile> {
        if services.is_empty() {
            return Err(EmptyIntlProfile);
        }
        let capabilities = services
            .iter()
            .fold(IntlCapabilitySet::EMPTY, |capabilities, service| {
                capabilities.union(service.required_capabilities())
            });
        Ok(Self {
            profile,
            services,
            capabilities,
        })
    }

    /// Adds only the data required by a sealed pure operation. This does not
    /// advertise a whole service whose other formatting data is absent.
    #[must_use]
    pub fn with_operation<O: IntlOperation>(mut self) -> Self {
        self.capabilities = self.capabilities.union(O::HOST_OP.required_capabilities());
        self
    }

    #[must_use]
    pub fn profile(&self) -> &IntlDataProfile {
        &self.profile
    }

    #[must_use]
    pub const fn services(&self) -> IntlServiceSet {
        self.services
    }

    #[must_use]
    pub const fn capabilities(&self) -> IntlCapabilitySet {
        self.capabilities
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyIntlProfile;

impl fmt::Display for EmptyIntlProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an Intl data profile must advertise at least one service")
    }
}

impl std::error::Error for EmptyIntlProfile {}

/// Immutable identity used to match a compiled artifact and runtime provider.
/// Current producers cannot choose a schema or data-version line: both are
/// fixed here and therefore cannot drift independently across call sites. The
/// default locale is canonical before it can enter this identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntlDataIdentity {
    schema: IntlDataSchemaVersion,
    profile: IntlProfilePlan,
    default_locale: CanonicalLocaleId,
    placement: IntlDataPlacement,
    digest: IntlDataDigest,
    versions: IntlDataVersions,
}

impl IntlDataIdentity {
    /// Creates the complete identity expected by one compiled artifact.
    ///
    /// A raw or merely structural locale cannot become the profile default.
    ///
    /// ```compile_fail
    /// use lila_intl::{
    ///     IntlDataDigest, IntlDataIdentity, IntlDataPlacement, IntlProfilePlan,
    /// };
    ///
    /// let _ = IntlDataIdentity::new(
    ///     IntlProfilePlan::conformance(),
    ///     "en-US",
    ///     IntlDataPlacement::Embedded,
    ///     IntlDataDigest::from_sha256([0; 32]),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        profile: IntlProfilePlan,
        default_locale: CanonicalLocaleId,
        placement: IntlDataPlacement,
        digest: IntlDataDigest,
    ) -> Self {
        Self {
            schema: INTL_DATA_SCHEMA_VERSION,
            profile,
            default_locale,
            placement,
            digest,
            versions: IntlDataVersions::PINNED,
        }
    }

    #[must_use]
    pub const fn schema(&self) -> IntlDataSchemaVersion {
        self.schema
    }

    #[must_use]
    pub const fn profile(&self) -> &IntlProfilePlan {
        &self.profile
    }

    #[must_use]
    pub const fn default_locale(&self) -> &CanonicalLocaleId {
        &self.default_locale
    }

    #[must_use]
    pub const fn placement(&self) -> IntlDataPlacement {
        self.placement
    }

    #[must_use]
    pub const fn digest(&self) -> IntlDataDigest {
        self.digest
    }

    #[must_use]
    pub const fn versions(&self) -> IntlDataVersions {
        self.versions
    }

    /// Serializes the complete identity for the versioned artifact section.
    ///
    /// The payload is canonical UTF-8: closed sets are sorted by their stable
    /// names, the digest is lowercase hexadecimal, and every identity field is
    /// present. Private construction of the return type prevents emitters from
    /// substituting a digest or profile fragment for the complete identity.
    #[must_use]
    pub fn artifact_identity(&self) -> IntlArtifactIdentity {
        let mut services = self
            .profile
            .services()
            .iter()
            .map(IntlService::name)
            .collect::<Vec<_>>();
        services.sort_unstable();
        let mut capabilities = self
            .profile
            .capabilities()
            .iter()
            .map(IntlDataCapability::name)
            .collect::<Vec<_>>();
        capabilities.sort_unstable();

        let profile = match self.profile.profile() {
            IntlDataProfile::Conformance => "conformance".to_string(),
            IntlDataProfile::Minimal => "minimal".to_string(),
            IntlDataProfile::Custom(id) => format!("custom:{}", id.as_str()),
        };
        let placement = match self.placement {
            IntlDataPlacement::Embedded => "embedded",
            IntlDataPlacement::External => "external",
        };
        let mut digest = String::with_capacity(64);
        for byte in self.digest.as_bytes() {
            write!(digest, "{byte:02x}").expect("writing to a String cannot fail");
        }

        let versions = self.versions;
        let bytes = format!(
            concat!(
                "schema={}\n",
                "host-call-abi={}\n",
                "profile={}\n",
                "services={}\n",
                "capabilities={}\n",
                "default-locale={}\n",
                "placement={}\n",
                "digest={}\n",
                "icu4x={}\n",
                "cldr={}\n",
                "unicode={}\n",
                "icu-data-tag={}\n",
                "segmenter-lstm={}\n",
                "tzdb={}\n",
            ),
            self.schema.get(),
            INTL_HOST_CALL_ABI_VERSION,
            profile,
            services.join(","),
            capabilities.join(","),
            self.default_locale.as_str(),
            placement,
            digest,
            versions.icu4x,
            versions.cldr,
            versions.unicode,
            versions.icu_data_tag,
            versions.segmenter_lstm,
            versions.tzdb,
        )
        .into_bytes()
        .into_boxed_slice();

        IntlArtifactIdentity(bytes)
    }
}

/// Canonical bytes for [`INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION`].
///
/// Construction is intentionally private: the only valid payload is the
/// serialization of a complete [`IntlDataIdentity`]. Consumers compare these
/// bytes directly, so the engine does not own a second identity decoder that
/// could drift from this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntlArtifactIdentity(Box<[u8]>);

impl IntlArtifactIdentity {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_profile_computes_capability_closure() {
        let services = IntlServiceSet::EMPTY.with(IntlService::DateTimeFormat);
        let plan = IntlProfilePlan::minimal(services).expect("one service is a profile");

        for capability in [
            IntlDataCapability::LocaleAliases,
            IntlDataCapability::Calendars,
            IntlDataCapability::DateTimePatterns,
            IntlDataCapability::TimeZoneNames,
            IntlDataCapability::TimeZoneTransitions,
        ] {
            assert!(plan.capabilities().contains(capability));
        }
        assert!(!plan.capabilities().contains(IntlDataCapability::Collation));
    }

    #[test]
    fn operation_data_does_not_advertise_a_complete_formatter_service() {
        let profile = IntlProfilePlan::minimal(IntlServiceSet::EMPTY.with(IntlService::Locale))
            .unwrap()
            .with_operation::<LookupNamedTimeZone>()
            .with_operation::<ResolveTimeZone>();
        assert_eq!(
            profile.services(),
            IntlServiceSet::EMPTY.with(IntlService::Locale)
        );
        assert!(profile
            .capabilities()
            .contains(IntlDataCapability::TimeZoneTransitions));
        assert!(profile
            .capabilities()
            .contains(IntlDataCapability::TimeZoneNames));
        assert!(!profile
            .capabilities()
            .contains(IntlDataCapability::DateTimePatterns));
        assert!(!profile
            .capabilities()
            .contains(IntlDataCapability::Calendars));
    }

    #[test]
    fn current_identity_fixes_schema_and_data_line() {
        let identity = IntlDataIdentity::new(
            IntlProfilePlan::conformance(),
            CanonicalLocaleId::from_data("en-US").expect("canonical default locale"),
            IntlDataPlacement::Embedded,
            IntlDataDigest::from_sha256([0x5a; 32]),
        );

        assert_eq!(identity.schema(), INTL_DATA_SCHEMA_VERSION);
        assert_eq!(identity.versions(), IntlDataVersions::PINNED);
        assert_eq!(identity.default_locale().as_str(), "en-US");
    }

    #[test]
    fn artifact_identity_v1_has_one_canonical_full_encoding() {
        let identity = IntlDataIdentity::new(
            IntlProfilePlan::minimal(IntlServiceSet::EMPTY.with(IntlService::Locale))
                .expect("one service is a profile"),
            CanonicalLocaleId::from_data("en-US").expect("canonical default locale"),
            IntlDataPlacement::External,
            IntlDataDigest::from_sha256([0x5a; 32]),
        );
        let artifact_identity = identity.artifact_identity();

        assert_eq!(
            core::str::from_utf8(artifact_identity.as_bytes())
                .expect("identity is canonical UTF-8"),
            concat!(
                "schema=1\n",
                "host-call-abi=5\n",
                "profile=minimal\n",
                "services=Locale\n",
                "capabilities=likely-subtags,locale-aliases,parent-locales\n",
                "default-locale=en-US\n",
                "placement=external\n",
                "digest=5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\n",
                "icu4x=2.0\n",
                "cldr=47.0.0\n",
                "unicode=16.0.0\n",
                "icu-data-tag=icu4x/2025-05-01/77.x\n",
                "segmenter-lstm=v0.1.0\n",
                "tzdb=2026a\n",
            )
        );
        assert_eq!(artifact_identity, identity.artifact_identity());
    }
}

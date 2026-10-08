//! Immutable host-selected zone, admitted by the same pinned kernel as queries.

use core::fmt;
use std::sync::{Arc, OnceLock};

use crate::{
    EmbeddedIntlProvider, EmbeddedIntlProviderSetupError, FixedTimeZoneOffset, IntlDataIdentity,
    IntlKernel, IntlProvider, IntlProviderIdentityMismatch, InvalidTimeZoneId, LookupNamedTimeZone,
    LookupNamedTimeZoneRequest, MissingIntlCapabilities, NamedTimeZoneIdentity,
    NamedTimeZoneLookupError, TimeZoneId,
};

#[derive(Debug)]
pub enum EmbeddedIntlKernelSetupError {
    Provider(EmbeddedIntlProviderSetupError),
    Identity(IntlProviderIdentityMismatch),
}

impl fmt::Display for EmbeddedIntlKernelSetupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider(error) => {
                write!(formatter, "embedded Intl provider setup failed: {error}")
            }
            Self::Identity(error) => {
                write!(formatter, "embedded Intl kernel setup failed: {error}")
            }
        }
    }
}
impl std::error::Error for EmbeddedIntlKernelSetupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Provider(error) => Some(error),
            Self::Identity(error) => Some(error),
        }
    }
}

/// The existing Engine cache has one owner; configuration and execution share it.
pub fn shared_embedded_intl_kernel(
) -> Result<Arc<IntlKernel<EmbeddedIntlProvider>>, Arc<EmbeddedIntlKernelSetupError>> {
    static KERNEL: OnceLock<
        Result<Arc<IntlKernel<EmbeddedIntlProvider>>, Arc<EmbeddedIntlKernelSetupError>>,
    > = OnceLock::new();
    KERNEL
        .get_or_init(|| {
            let provider = EmbeddedIntlProvider::new()
                .map_err(|error| Arc::new(EmbeddedIntlKernelSetupError::Provider(error)))?;
            let expected_identity = provider.identity().clone();
            IntlKernel::new(expected_identity, provider)
                .map(Arc::new)
                .map_err(|error| Arc::new(EmbeddedIntlKernelSetupError::Identity(error)))
        })
        .clone()
}

/// Shared wire vocabulary for the configured Realm zone and emitted proofs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SystemTimeZoneKind {
    Utc = 0,
    FixedOffset = 1,
    Named = 2,
}
impl SystemTimeZoneKind {
    pub const fn code(self) -> i64 {
        self as i64
    }
}

pub const SYSTEM_TIME_ZONE_WIRE_VERSION: i64 = 1;
pub const SYSTEM_TIME_ZONE_HEADER_BYTES: usize = 32;
pub const SYSTEM_TIME_ZONE_VERSION_OFFSET: u64 = 0;
pub const SYSTEM_TIME_ZONE_KIND_OFFSET: u64 = 8;
pub const SYSTEM_TIME_ZONE_FIXED_SECONDS_OFFSET: u64 = 16;
pub const SYSTEM_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET: u64 = 24;

#[derive(Debug, Clone)]
enum SystemTimeZoneChoice {
    Utc,
    Fixed {
        identifier: Box<str>,
        offset: FixedTimeZoneOffset,
    },
    Named {
        identity: NamedTimeZoneIdentity,
        provider_identity: IntlDataIdentity,
    },
}

/// No public constructor can attach an arbitrary string to a provider proof.
#[derive(Debug, Clone)]
pub struct ConfiguredSystemTimeZone {
    choice: SystemTimeZoneChoice,
}

impl ConfiguredSystemTimeZone {
    pub const fn utc() -> Self {
        Self {
            choice: SystemTimeZoneChoice::Utc,
        }
    }

    pub fn resolve(identifier: &str) -> Result<Self, SystemTimeZoneConfigurationError> {
        let identifier = TimeZoneId::parse(identifier)
            .map_err(SystemTimeZoneConfigurationError::InvalidIdentifier)?;
        if identifier.as_str().eq_ignore_ascii_case("UTC") {
            return Ok(Self::utc());
        }
        let bytes = identifier.as_str().as_bytes();
        if matches!(bytes[0], b'+' | b'-') {
            // TimeZoneId has proved the exact six-byte bounded minute grammar.
            let hours = i64::from((bytes[1] - b'0') * 10 + bytes[2] - b'0');
            let minutes = i64::from((bytes[4] - b'0') * 10 + bytes[5] - b'0');
            let unsigned = hours * 3_600 + minutes * 60;
            let seconds = if bytes[0] == b'-' {
                -unsigned
            } else {
                unsigned
            };
            let offset = FixedTimeZoneOffset::from_seconds(seconds)
                .expect("validated minute grammar proves fixed offset range");
            let sign = if seconds < 0 { '-' } else { '+' };
            let identifier = format!("{sign}{hours:02}:{minutes:02}").into_boxed_str();
            return Ok(Self {
                choice: SystemTimeZoneChoice::Fixed { identifier, offset },
            });
        }
        let kernel =
            shared_embedded_intl_kernel().map_err(SystemTimeZoneConfigurationError::KernelSetup)?;
        let identity = kernel
            .operation::<LookupNamedTimeZone>()
            .map_err(SystemTimeZoneConfigurationError::Capability)?
            .execute(LookupNamedTimeZoneRequest::new(identifier))
            .map_err(SystemTimeZoneConfigurationError::Lookup)?
            .identity()
            .clone();
        Ok(Self {
            choice: SystemTimeZoneChoice::Named {
                identity,
                provider_identity: kernel.identity().clone(),
            },
        })
    }

    pub fn primary_identifier(&self) -> &str {
        match &self.choice {
            SystemTimeZoneChoice::Utc => "UTC",
            SystemTimeZoneChoice::Fixed { identifier, .. } => identifier,
            SystemTimeZoneChoice::Named { identity, .. } => identity.primary_identifier(),
        }
    }

    pub fn kind(&self) -> SystemTimeZoneKind {
        match &self.choice {
            SystemTimeZoneChoice::Utc => SystemTimeZoneKind::Utc,
            SystemTimeZoneChoice::Fixed { .. } => SystemTimeZoneKind::FixedOffset,
            SystemTimeZoneChoice::Named { identity, .. }
                if identity.primary_identifier() == "UTC" =>
            {
                SystemTimeZoneKind::Utc
            }
            SystemTimeZoneChoice::Named { .. } => SystemTimeZoneKind::Named,
        }
    }

    pub fn fixed_offset_seconds(&self) -> i64 {
        match &self.choice {
            SystemTimeZoneChoice::Fixed { offset, .. } => i64::from(offset.seconds()),
            SystemTimeZoneChoice::Utc | SystemTimeZoneChoice::Named { .. } => 0,
        }
    }

    pub fn is_compatible_with(&self, actual: &IntlDataIdentity) -> bool {
        match &self.choice {
            SystemTimeZoneChoice::Named {
                provider_identity, ..
            } => provider_identity == actual,
            SystemTimeZoneChoice::Utc | SystemTimeZoneChoice::Fixed { .. } => true,
        }
    }

    /// Publishes only the primary name. The admitted alias remains private.
    pub fn encode(&self) -> Vec<u8> {
        let identifier = self.primary_identifier().as_bytes();
        let mut bytes = Vec::with_capacity(SYSTEM_TIME_ZONE_HEADER_BYTES + identifier.len());
        for word in [
            SYSTEM_TIME_ZONE_WIRE_VERSION,
            self.kind().code(),
            self.fixed_offset_seconds(),
            identifier.len() as i64,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(identifier);
        bytes
    }
}

#[derive(Debug)]
pub enum SystemTimeZoneConfigurationError {
    InvalidIdentifier(InvalidTimeZoneId),
    Lookup(NamedTimeZoneLookupError),
    KernelSetup(Arc<EmbeddedIntlKernelSetupError>),
    Capability(MissingIntlCapabilities),
}
impl fmt::Display for SystemTimeZoneConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier(error) => error.fmt(formatter),
            Self::Lookup(error) => error.fmt(formatter),
            Self::KernelSetup(error) => error.fmt(formatter),
            Self::Capability(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for SystemTimeZoneConfigurationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidIdentifier(error) => Some(error),
            Self::Lookup(error) => Some(error),
            Self::KernelSetup(error) => Some(&**error),
            Self::Capability(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IntlDataDigest;

    fn response_words(zone: &ConfiguredSystemTimeZone) -> ([i64; 4], String) {
        let bytes = zone.encode();
        let words = std::array::from_fn(|index| {
            i64::from_le_bytes(bytes[index * 8..index * 8 + 8].try_into().unwrap())
        });
        let identifier =
            String::from_utf8(bytes[SYSTEM_TIME_ZONE_HEADER_BYTES..].to_vec()).unwrap();
        assert_eq!(words[3] as usize, identifier.len());
        (words, identifier)
    }

    #[test]
    fn utc_default_has_a_single_primary_response() {
        let zone = ConfiguredSystemTimeZone::utc();
        assert_eq!(response_words(&zone), ([1, 0, 0, 3], "UTC".to_owned()));
        assert_eq!(
            ConfiguredSystemTimeZone::resolve("uTc")
                .unwrap()
                .primary_identifier(),
            "UTC"
        );
    }

    #[test]
    fn numeric_zero_remains_fixed_and_normalizes_negative_zero() {
        for input in ["+00:00", "-00:00"] {
            let zone = ConfiguredSystemTimeZone::resolve(input).unwrap();
            assert_eq!(response_words(&zone), ([1, 1, 0, 6], "+00:00".to_owned()));
            assert_ne!(zone.kind(), ConfiguredSystemTimeZone::utc().kind());
        }
    }

    #[test]
    fn admitted_fixed_offsets_cover_both_extremes_and_half_hours() {
        for (input, seconds) in [
            ("+23:59", 86_340),
            ("-23:59", -86_340),
            ("+05:30", 19_800),
            ("-03:30", -12_600),
        ] {
            let zone = ConfiguredSystemTimeZone::resolve(input).unwrap();
            assert_eq!(
                response_words(&zone),
                ([1, 1, seconds, 6], input.to_owned())
            );
        }
    }

    #[test]
    fn invalid_configuration_never_becomes_a_runtime_zone() {
        for input in [
            "",
            "+24:00",
            "+01:60",
            "+0100",
            "+01:02:03",
            "Europe/../Paris",
            "Europe//Paris",
            "Europé/Paris",
        ] {
            assert!(
                matches!(
                    ConfiguredSystemTimeZone::resolve(input),
                    Err(SystemTimeZoneConfigurationError::InvalidIdentifier(_))
                ),
                "{input}"
            );
        }
        assert!(matches!(
            ConfiguredSystemTimeZone::resolve("Not_A_Real_Zone"),
            Err(SystemTimeZoneConfigurationError::Lookup(
                NamedTimeZoneLookupError::Unknown(_)
            ))
        ));
    }

    #[test]
    fn named_links_publish_primary_and_preserve_internal_admission() {
        let zone = ConfiguredSystemTimeZone::resolve("us/eastern").unwrap();
        assert_eq!(
            response_words(&zone),
            ([1, 2, 0, 16], "America/New_York".to_owned())
        );
        match &zone.choice {
            SystemTimeZoneChoice::Named { identity, .. } => {
                assert_eq!(identity.identifier(), "US/Eastern")
            }
            SystemTimeZoneChoice::Utc | SystemTimeZoneChoice::Fixed { .. } => {
                panic!("named admission was lost")
            }
        }
    }

    #[test]
    fn utc_links_publish_the_canonical_utc_association() {
        let zone = ConfiguredSystemTimeZone::resolve("Etc/UTC").unwrap();
        assert_eq!(response_words(&zone), ([1, 0, 0, 3], "UTC".to_owned()));
    }

    #[test]
    fn named_configuration_is_bound_to_actual_provider_identity() {
        let kernel = shared_embedded_intl_kernel().unwrap();
        let zone = ConfiguredSystemTimeZone::resolve("Europe/Paris").unwrap();
        assert!(zone.is_compatible_with(kernel.identity()));
        let actual = kernel.identity();
        let changed = IntlDataIdentity::new(
            actual.profile().clone(),
            actual.default_locale().clone(),
            actual.placement(),
            IntlDataDigest::from_sha256([0; 32]),
        );
        assert!(!zone.is_compatible_with(&changed));
        assert!(ConfiguredSystemTimeZone::utc().is_compatible_with(&changed));
        assert!(ConfiguredSystemTimeZone::resolve("+00:00")
            .unwrap()
            .is_compatible_with(&changed));
    }

    #[test]
    fn configuration_and_execution_share_one_kernel_instance() {
        let first = shared_embedded_intl_kernel().unwrap();
        ConfiguredSystemTimeZone::resolve("Europe/Paris").unwrap();
        let second = shared_embedded_intl_kernel().unwrap();
        assert!(Arc::ptr_eq(&first, &second));
    }
}

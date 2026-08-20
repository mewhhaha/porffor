use core::{fmt, marker::PhantomData};

use crate::{
    CanonicalLocaleId, CanonicalTimeZoneId, IntlCapabilitySet, IntlDataCapability,
    IntlDataIdentity, InvalidCanonicalLocaleId, LocaleId, TimeZoneId,
};

/// Packed offset/length span read by an Intl host operation.
///
/// Both halves are unsigned 32-bit values and every `i64` bit pattern therefore
/// decodes to exactly one span. Keeping read spans distinct from write spans
/// prevents a request length from being mistaken for output capacity in host
/// bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntlHostReadSpan(u64);

impl IntlHostReadSpan {
    #[must_use]
    pub const fn new(offset: u32, length: u32) -> Self {
        Self(((offset as u64) << 32) | length as u64)
    }

    #[must_use]
    pub const fn from_wire(wire: i64) -> Self {
        Self(wire as u64)
    }

    #[must_use]
    pub const fn wire(self) -> i64 {
        self.0 as i64
    }

    #[must_use]
    pub const fn offset(self) -> u32 {
        (self.0 >> 32) as u32
    }

    #[must_use]
    pub const fn length(self) -> u32 {
        self.0 as u32
    }
}

/// Packed offset/capacity span written by an Intl host operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntlHostWriteSpan(u64);

impl IntlHostWriteSpan {
    #[must_use]
    pub const fn new(offset: u32, capacity: u32) -> Self {
        Self(((offset as u64) << 32) | capacity as u64)
    }

    #[must_use]
    pub const fn from_wire(wire: i64) -> Self {
        Self(wire as u64)
    }

    #[must_use]
    pub const fn wire(self) -> i64 {
        self.0 as i64
    }

    #[must_use]
    pub const fn offset(self) -> u32 {
        (self.0 >> 32) as u32
    }

    #[must_use]
    pub const fn capacity(self) -> u32 {
        self.0 as u32
    }
}

/// Closed result domain for the shared `(op, request_span, result_span) -> i64`
/// host ABI.
///
/// Expected provider rejection is the sole negative value. All non-negative
/// `u32` values are successful byte counts; every other `i64` is an ABI fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntlHostCallOutcome {
    Written(u32),
    Rejected,
}

impl IntlHostCallOutcome {
    pub const REJECTED_WIRE: i64 = -1;

    #[must_use]
    pub const fn wire(self) -> i64 {
        match self {
            Self::Written(length) => length as i64,
            Self::Rejected => Self::REJECTED_WIRE,
        }
    }

    #[must_use]
    pub const fn from_wire(wire: i64) -> Option<Self> {
        if wire == Self::REJECTED_WIRE {
            Some(Self::Rejected)
        } else if wire >= 0 && wire <= u32::MAX as i64 {
            Some(Self::Written(wire as u32))
        } else {
            None
        }
    }
}

macro_rules! intl_operations {
    (
        $(
            $operation:ident {
                code: $code:literal,
                name: $name:literal,
                request: $request:ty,
                response: $response:ty,
                error: $error:ty,
                capabilities: [$($capability:path),* $(,)?],
            }
        )+
    ) => {
        /// Stable operation tags for the provider-independent Intl kernel
        /// boundary. The same row also generates the only operation marker
        /// that can carry each request/result/error association.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u16)]
        pub enum IntlHostOp {
            $($operation = $code),+
        }

        impl IntlHostOp {
            pub const ALL: &'static [Self] = &[$(Self::$operation),+];

            #[must_use]
            pub const fn code(self) -> u16 {
                self as u16
            }

            #[must_use]
            pub const fn wire(self) -> i64 {
                self as i64
            }

            #[must_use]
            pub const fn from_wire(wire: i64) -> Option<Self> {
                match wire {
                    $($code => Some(Self::$operation),)+
                    _ => None,
                }
            }

            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$operation => $name),+
                }
            }

            #[must_use]
            pub const fn required_capabilities(self) -> IntlCapabilitySet {
                match self {
                    $(
                        Self::$operation => IntlCapabilitySet::EMPTY
                            $(.with($capability))*,
                    )+
                }
            }
        }

        $(
            #[derive(Debug)]
            pub enum $operation {}

            impl sealed::Sealed for $operation {}

            impl IntlOperation for $operation {
                type Request = $request;
                type Response = $response;
                type Error = $error;

                const HOST_OP: IntlHostOp = IntlHostOp::$operation;
            }
        )+

        const _: () = {
            let mut mask = 0u16;
            let mut index = 0;
            while index < IntlHostOp::ALL.len() {
                mask |= 1u16 << IntlHostOp::ALL[index] as u16;
                assert!(!IntlHostOp::ALL[index]
                    .required_capabilities()
                    .is_empty());
                index += 1;
            }
            assert!(mask == (1u16 << IntlHostOp::ALL.len()) - 1);
        };
    };
}

mod sealed {
    pub trait Sealed {}
}

/// A closed association between one host operation and its request, response,
/// failure and data-capability contract.
pub trait IntlOperation: sealed::Sealed {
    type Request;
    type Response;
    type Error: std::error::Error + Send + Sync + 'static;

    const HOST_OP: IntlHostOp;
}

intl_operations! {
    CanonicalizeLocale {
        code: 0,
        name: "canonicalize-locale",
        request: CanonicalizeLocaleRequest,
        response: CanonicalizeLocaleResult,
        error: CanonicalizeLocaleError,
        capabilities: [IntlDataCapability::LocaleAliases],
    }
    CanonicalizeTimeZone {
        code: 1,
        name: "canonicalize-time-zone",
        request: CanonicalizeTimeZoneRequest,
        response: CanonicalizeTimeZoneResult,
        error: UnknownTimeZone,
        capabilities: [IntlDataCapability::TimeZoneTransitions],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalizeLocaleRequest {
    locale: LocaleId,
}

impl CanonicalizeLocaleRequest {
    /// Creates a request from a structurally validated locale identifier.
    ///
    /// Raw strings cannot cross the kernel boundary.
    ///
    /// ```compile_fail
    /// use lila_intl::CanonicalizeLocaleRequest;
    ///
    /// let _ = CanonicalizeLocaleRequest::new("en-US");
    /// ```
    #[must_use]
    pub const fn new(locale: LocaleId) -> Self {
        Self { locale }
    }

    #[must_use]
    pub const fn locale(&self) -> &LocaleId {
        &self.locale
    }

    #[must_use]
    pub fn into_locale(self) -> LocaleId {
        self.locale
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalizeLocaleResult {
    locale: CanonicalLocaleId,
}

impl CanonicalizeLocaleResult {
    #[must_use]
    pub const fn new(locale: CanonicalLocaleId) -> Self {
        Self { locale }
    }

    #[must_use]
    pub const fn locale(&self) -> &CanonicalLocaleId {
        &self.locale
    }

    #[must_use]
    pub fn into_locale(self) -> CanonicalLocaleId {
        self.locale
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalizeTimeZoneRequest {
    time_zone: TimeZoneId,
}

impl CanonicalizeTimeZoneRequest {
    #[must_use]
    pub const fn new(time_zone: TimeZoneId) -> Self {
        Self { time_zone }
    }

    #[must_use]
    pub const fn time_zone(&self) -> &TimeZoneId {
        &self.time_zone
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalizeTimeZoneResult {
    time_zone: CanonicalTimeZoneId,
}

impl CanonicalizeTimeZoneResult {
    #[must_use]
    pub const fn new(time_zone: CanonicalTimeZoneId) -> Self {
        Self { time_zone }
    }

    #[must_use]
    pub const fn time_zone(&self) -> &CanonicalTimeZoneId {
        &self.time_zone
    }

    #[must_use]
    pub fn into_time_zone(self) -> CanonicalTimeZoneId {
        self.time_zone
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedLocale {
    locale: LocaleId,
}

impl UnsupportedLocale {
    #[must_use]
    pub const fn new(locale: LocaleId) -> Self {
        Self { locale }
    }

    #[must_use]
    pub const fn locale(&self) -> &LocaleId {
        &self.locale
    }
}

impl fmt::Display for UnsupportedLocale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Intl provider does not support locale {:?}",
            self.locale.as_str()
        )
    }
}

impl std::error::Error for UnsupportedLocale {}

/// Provider result for locale canonicalization, separating an expected
/// unsupported input from corrupt/non-canonical provider output.
#[derive(Debug)]
pub enum CanonicalizeLocaleError {
    Unsupported(UnsupportedLocale),
    InvalidProviderResult(InvalidCanonicalLocaleId),
}

impl fmt::Display for CanonicalizeLocaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(error) => error.fmt(f),
            Self::InvalidProviderResult(error) => {
                write!(
                    f,
                    "Intl provider returned invalid canonical locale: {error}"
                )
            }
        }
    }
}

impl std::error::Error for CanonicalizeLocaleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unsupported(error) => Some(error),
            Self::InvalidProviderResult(error) => Some(error),
        }
    }
}

impl From<UnsupportedLocale> for CanonicalizeLocaleError {
    fn from(error: UnsupportedLocale) -> Self {
        Self::Unsupported(error)
    }
}

impl From<InvalidCanonicalLocaleId> for CanonicalizeLocaleError {
    fn from(error: InvalidCanonicalLocaleId) -> Self {
        Self::InvalidProviderResult(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTimeZone {
    time_zone: TimeZoneId,
}

impl UnknownTimeZone {
    #[must_use]
    pub const fn new(time_zone: TimeZoneId) -> Self {
        Self { time_zone }
    }

    #[must_use]
    pub const fn time_zone(&self) -> &TimeZoneId {
        &self.time_zone
    }
}

impl fmt::Display for UnknownTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Intl provider does not know time zone {:?}",
            self.time_zone.as_str()
        )
    }
}

impl std::error::Error for UnknownTimeZone {}

/// Immutable identity exposed by a pure data provider.
pub trait IntlProvider {
    fn identity(&self) -> &IntlDataIdentity;
}

/// Provider implementation for one closed operation.
///
/// Implementations never receive JavaScript values: only the request type
/// associated with `O` can reach this method.
pub trait IntlOperationProvider<O: IntlOperation>: IntlProvider {
    fn execute(&self, request: O::Request) -> Result<O::Response, O::Error>;
}

/// An identity-matched provider ready to grant typed operation capabilities.
///
/// A profile plan alone is deliberately not sufficient to instantiate the
/// kernel; artifact schema, versions, placement and digest must all match.
///
/// ```compile_fail
/// use lila_intl::{IntlKernel, IntlProfilePlan, IntlProvider};
///
/// fn install<P: IntlProvider>(provider: P) {
///     let _ = IntlKernel::new(IntlProfilePlan::conformance(), provider);
/// }
/// ```
#[derive(Debug)]
pub struct IntlKernel<P> {
    identity: IntlDataIdentity,
    provider: P,
}

impl<P: IntlProvider> IntlKernel<P> {
    pub fn new(
        expected: IntlDataIdentity,
        provider: P,
    ) -> Result<Self, IntlProviderIdentityMismatch> {
        let actual: IntlDataIdentity = provider.identity().clone();
        if actual != expected {
            return Err(IntlProviderIdentityMismatch { expected, actual });
        }
        Ok(Self {
            identity: expected,
            provider,
        })
    }

    #[must_use]
    pub const fn identity(&self) -> &IntlDataIdentity {
        &self.identity
    }

    pub fn operation<O>(&self) -> Result<IntlOperationHandle<'_, P, O>, MissingIntlCapabilities>
    where
        O: IntlOperation,
        P: IntlOperationProvider<O>,
    {
        let available = self.identity.profile().capabilities();
        let required = O::HOST_OP.required_capabilities();
        if !available.contains_all(required) {
            return Err(MissingIntlCapabilities {
                operation: O::HOST_OP,
                missing: required.difference(available),
            });
        }
        Ok(IntlOperationHandle {
            kernel: self,
            operation: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct IntlOperationHandle<'a, P, O> {
    kernel: &'a IntlKernel<P>,
    operation: PhantomData<fn() -> O>,
}

impl<P, O> IntlOperationHandle<'_, P, O>
where
    O: IntlOperation,
    P: IntlOperationProvider<O>,
{
    /// Executes the one operation authorized by this handle.
    ///
    /// A locale request cannot be sent through a time-zone handle (or vice
    /// versa), because the request type is selected by `O`.
    ///
    /// ```compile_fail
    /// use lila_intl::{
    ///     CanonicalizeLocale, CanonicalizeTimeZoneRequest, IntlOperationHandle,
    ///     IntlOperationProvider, TimeZoneId,
    /// };
    ///
    /// fn wrong_request<P>(handle: IntlOperationHandle<'_, P, CanonicalizeLocale>)
    /// where
    ///     P: IntlOperationProvider<CanonicalizeLocale>,
    /// {
    ///     let zone = TimeZoneId::parse("UTC").unwrap();
    ///     let _ = handle.execute(CanonicalizeTimeZoneRequest::new(zone));
    /// }
    /// ```
    pub fn execute(&self, request: O::Request) -> Result<O::Response, O::Error> {
        IntlOperationProvider::<O>::execute(&self.kernel.provider, request)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntlProviderIdentityMismatch {
    expected: IntlDataIdentity,
    actual: IntlDataIdentity,
}

impl IntlProviderIdentityMismatch {
    #[must_use]
    pub const fn expected(&self) -> &IntlDataIdentity {
        &self.expected
    }

    #[must_use]
    pub const fn actual(&self) -> &IntlDataIdentity {
        &self.actual
    }
}

impl fmt::Display for IntlProviderIdentityMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Intl provider identity does not match the artifact identity")
    }
}

impl std::error::Error for IntlProviderIdentityMismatch {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingIntlCapabilities {
    operation: IntlHostOp,
    missing: IntlCapabilitySet,
}

impl MissingIntlCapabilities {
    #[must_use]
    pub const fn operation(&self) -> IntlHostOp {
        self.operation
    }

    #[must_use]
    pub const fn missing(&self) -> IntlCapabilitySet {
        self.missing
    }
}

impl fmt::Display for MissingIntlCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Intl operation {} requires", self.operation.name())?;
        for capability in self.missing.iter() {
            write!(f, " {}", capability.name())?;
        }
        Ok(())
    }
}

impl std::error::Error for MissingIntlCapabilities {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntlDataDigest, IntlDataPlacement, IntlProfilePlan, IntlService, IntlServiceSet};

    #[derive(Debug)]
    struct FixtureProvider {
        identity: IntlDataIdentity,
    }

    impl IntlProvider for FixtureProvider {
        fn identity(&self) -> &IntlDataIdentity {
            &self.identity
        }
    }

    impl IntlOperationProvider<CanonicalizeLocale> for FixtureProvider {
        fn execute(
            &self,
            request: CanonicalizeLocaleRequest,
        ) -> Result<CanonicalizeLocaleResult, CanonicalizeLocaleError> {
            let locale = match request.locale().as_str() {
                "EN-us" => CanonicalLocaleId::from_data("en-US").unwrap(),
                "iw-IL" => CanonicalLocaleId::from_data("he-IL").unwrap(),
                _ => return Err(UnsupportedLocale::new(request.locale).into()),
            };
            Ok(CanonicalizeLocaleResult::new(locale))
        }
    }

    impl IntlOperationProvider<CanonicalizeTimeZone> for FixtureProvider {
        fn execute(
            &self,
            request: CanonicalizeTimeZoneRequest,
        ) -> Result<CanonicalizeTimeZoneResult, UnknownTimeZone> {
            let time_zone = match request.time_zone().as_str() {
                "europe/stockholm" => CanonicalTimeZoneId::from_data("Europe/Stockholm").unwrap(),
                "Etc/UTC" => CanonicalTimeZoneId::from_data("UTC").unwrap(),
                _ => return Err(UnknownTimeZone::new(request.time_zone)),
            };
            Ok(CanonicalizeTimeZoneResult::new(time_zone))
        }
    }

    fn identity_for(services: IntlServiceSet, digest: u8) -> IntlDataIdentity {
        IntlDataIdentity::new(
            IntlProfilePlan::minimal(services).unwrap(),
            CanonicalLocaleId::from_data("en-US").unwrap(),
            IntlDataPlacement::External,
            IntlDataDigest::from_sha256([digest; 32]),
        )
    }

    #[test]
    fn kernel_consumes_typed_locale_and_time_zone_paths() {
        let identity = IntlDataIdentity::new(
            IntlProfilePlan::conformance(),
            CanonicalLocaleId::from_data("en-US").unwrap(),
            IntlDataPlacement::Embedded,
            IntlDataDigest::from_sha256([0x23; 32]),
        );
        let provider = FixtureProvider {
            identity: identity.clone(),
        };
        let kernel = IntlKernel::new(identity, provider).unwrap();

        let locale = kernel
            .operation::<CanonicalizeLocale>()
            .unwrap()
            .execute(CanonicalizeLocaleRequest::new(
                LocaleId::parse("iw-IL").unwrap(),
            ))
            .unwrap();
        assert_eq!(locale.locale().as_str(), "he-IL");

        let time_zone = kernel
            .operation::<CanonicalizeTimeZone>()
            .unwrap()
            .execute(CanonicalizeTimeZoneRequest::new(
                TimeZoneId::parse("europe/stockholm").unwrap(),
            ))
            .unwrap();
        assert_eq!(time_zone.time_zone().as_str(), "Europe/Stockholm");
    }

    #[test]
    fn profile_capabilities_are_checked_before_provider_execution() {
        let services = IntlServiceSet::EMPTY.with(IntlService::Locale);
        let identity = identity_for(services, 0x11);
        let provider = FixtureProvider {
            identity: identity.clone(),
        };
        let kernel = IntlKernel::new(identity, provider).unwrap();

        assert!(kernel.operation::<CanonicalizeLocale>().is_ok());
        let error = kernel.operation::<CanonicalizeTimeZone>().unwrap_err();
        assert_eq!(error.operation(), IntlHostOp::CanonicalizeTimeZone);
        assert!(error
            .missing()
            .contains(IntlDataCapability::TimeZoneTransitions));
    }

    #[test]
    fn provider_identity_mismatch_rejects_kernel_construction() {
        let services = IntlServiceSet::EMPTY.with(IntlService::Locale);
        let expected = identity_for(services, 0x11);
        let provider = FixtureProvider {
            identity: identity_for(services, 0x12),
        };

        assert!(IntlKernel::new(expected, provider).is_err());
    }

    #[test]
    fn intl_host_wire_domain_is_closed_and_stable() {
        assert_eq!(IntlHostOp::CanonicalizeLocale.wire(), 0);
        assert_eq!(IntlHostOp::CanonicalizeTimeZone.wire(), 1);
        assert_eq!(
            IntlHostOp::from_wire(0),
            Some(IntlHostOp::CanonicalizeLocale)
        );
        assert_eq!(
            IntlHostOp::from_wire(1),
            Some(IntlHostOp::CanonicalizeTimeZone)
        );
        assert_eq!(IntlHostOp::from_wire(-1), None);
        assert_eq!(IntlHostOp::from_wire(2), None);

        let read = IntlHostReadSpan::new(u32::MAX, u32::MAX);
        assert_eq!(IntlHostReadSpan::from_wire(read.wire()), read);
        assert_eq!(read.offset(), u32::MAX);
        assert_eq!(read.length(), u32::MAX);

        let write = IntlHostWriteSpan::new(u32::MAX, u32::MAX);
        assert_eq!(IntlHostWriteSpan::from_wire(write.wire()), write);
        assert_eq!(write.offset(), u32::MAX);
        assert_eq!(write.capacity(), u32::MAX);

        for outcome in [
            IntlHostCallOutcome::Rejected,
            IntlHostCallOutcome::Written(0),
            IntlHostCallOutcome::Written(u32::MAX),
        ] {
            assert_eq!(
                IntlHostCallOutcome::from_wire(outcome.wire()),
                Some(outcome)
            );
        }
        assert_eq!(IntlHostCallOutcome::from_wire(-2), None);
        assert_eq!(IntlHostCallOutcome::from_wire(u32::MAX as i64 + 1), None);
    }
}

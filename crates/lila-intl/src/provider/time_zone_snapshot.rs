use crate::{FixedTimeZoneOffset, NamedTimeZoneIdentity, TimeZoneEpochSeconds};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TimeZoneVariant {
    Standard,
    Daylight,
}

/// LDML47 Type Fallback can use a standard name only with a complete window proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StandardTimeStability {
    Stable,
    NotProven,
}

pub(super) const STANDARD_TIME_STABILITY_WINDOW_SECONDS: i64 = 184 * 86_400;

/// Offset and variant from the same selected IANA rearguard transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NamedTimeZoneTransition {
    offset_seconds: i32,
    variant: TimeZoneVariant,
    standard_time_stability: StandardTimeStability,
}

impl NamedTimeZoneTransition {
    pub(super) const fn new(offset_seconds: i32, variant: TimeZoneVariant) -> Self {
        Self {
            offset_seconds,
            variant,
            standard_time_stability: StandardTimeStability::NotProven,
        }
    }
    /// Only the complete transition-window query constructs this proof.
    pub(super) const fn stable_standard(offset_seconds: i32) -> Self {
        Self {
            offset_seconds,
            variant: TimeZoneVariant::Standard,
            standard_time_stability: StandardTimeStability::Stable,
        }
    }
    pub(super) const fn offset_seconds(self) -> i32 {
        self.offset_seconds
    }
    pub(super) const fn variant(self) -> TimeZoneVariant {
        self.variant
    }
    pub(super) const fn standard_time_stability(self) -> StandardTimeStability {
        self.standard_time_stability
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum TimeZoneNameInput<'a> {
    Named {
        identity: &'a NamedTimeZoneIdentity,
        epoch: TimeZoneEpochSeconds,
        transition: NamedTimeZoneTransition,
    },
    FixedOffset(FixedTimeZoneOffset),
}

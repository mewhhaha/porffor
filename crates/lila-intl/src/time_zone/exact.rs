//! Exact data coordinates. These types never apply Temporal option policy.

use super::{InvalidTimeZoneData, InvalidTimeZoneRequest};

const NANOS_PER_SECOND: i128 = 1_000_000_000;
const SECONDS_PER_DAY: i64 = 86_400;
const INSTANT_LIMIT_SECONDS: i64 = super::TimeZoneEpochSeconds::LIMIT;

/// An exact Instant-domain coordinate, normalized with Euclidean division.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeZoneInstant {
    seconds: i64,
    nanosecond: u32,
}

impl TimeZoneInstant {
    pub fn from_nanoseconds(value: i128) -> Result<Self, InvalidTimeZoneRequest> {
        let (seconds, nanosecond) = split(value)?;
        Self::new(seconds, nanosecond)
    }

    pub fn new(seconds: i64, nanosecond: u32) -> Result<Self, InvalidTimeZoneRequest> {
        if nanosecond >= 1_000_000_000
            || !(-INSTANT_LIMIT_SECONDS..=INSTANT_LIMIT_SECONDS).contains(&seconds)
            || (seconds == INSTANT_LIMIT_SECONDS && nanosecond != 0)
        {
            return Err(InvalidTimeZoneRequest("invalid exact Instant coordinate"));
        }
        Ok(Self {
            seconds,
            nanosecond,
        })
    }

    pub const fn seconds(self) -> i64 {
        self.seconds
    }
    pub const fn nanosecond(self) -> u32 {
        self.nanosecond
    }
    pub const fn nanoseconds(self) -> i128 {
        self.seconds as i128 * NANOS_PER_SECOND + self.nanosecond as i128
    }
    pub(crate) const fn limit_seconds() -> i64 {
        INSTANT_LIMIT_SECONDS
    }
}

/// ISO-local time before zone interpretation. Its domain includes the entire
/// PlainDateTime range and one additional day for a gap adjustment. This is
/// deliberately wider than Instant; Wasm owns the prescribed JS range checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTimeCoordinate {
    seconds: i64,
    nanosecond: u32,
}

impl LocalTimeCoordinate {
    pub fn from_nanoseconds(value: i128) -> Result<Self, InvalidTimeZoneRequest> {
        let (seconds, nanosecond) = split(value)?;
        Self::new(seconds, nanosecond)
    }
    pub fn new(seconds: i64, nanosecond: u32) -> Result<Self, InvalidTimeZoneRequest> {
        let limit = INSTANT_LIMIT_SECONDS + 2 * SECONDS_PER_DAY;
        if nanosecond >= 1_000_000_000 || !(-limit..=limit).contains(&seconds) {
            return Err(InvalidTimeZoneRequest(
                "invalid contextual ISO-local coordinate",
            ));
        }
        Ok(Self {
            seconds,
            nanosecond,
        })
    }
    pub const fn seconds(self) -> i64 {
        self.seconds
    }
    pub const fn nanosecond(self) -> u32 {
        self.nanosecond
    }
    pub const fn nanoseconds(self) -> i128 {
        self.seconds as i128 * NANOS_PER_SECOND + self.nanosecond as i128
    }
}

/// A possible epoch, including epochs outside the Instant range. No implicit
/// conversion to TimeZoneInstant exists: every candidate must be range checked
/// in Wasm before disambiguation, including candidates it will not select.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawTimeZoneEpoch {
    seconds: i64,
    nanosecond: u32,
}
impl RawTimeZoneEpoch {
    pub(crate) fn from_nanoseconds(value: i128) -> Result<Self, InvalidTimeZoneData> {
        let (seconds, nanosecond) =
            split(value).map_err(|_| InvalidTimeZoneData("candidate epoch overflow"))?;
        let limit = INSTANT_LIMIT_SECONDS + 3 * SECONDS_PER_DAY;
        if !(-limit..=limit).contains(&seconds) {
            return Err(InvalidTimeZoneData("candidate exceeds provider context"));
        }
        Ok(Self {
            seconds,
            nanosecond,
        })
    }
    pub const fn seconds(self) -> i64 {
        self.seconds
    }
    pub const fn nanosecond(self) -> u32 {
        self.nanosecond
    }
    pub const fn nanoseconds(self) -> i128 {
        self.seconds as i128 * NANOS_PER_SECOND + self.nanosecond as i128
    }
}

/// The actual selected UTC offset, with the Temporal less-than-one-day proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NamedTimeZoneOffsetSeconds(i32);
impl NamedTimeZoneOffsetSeconds {
    /// Shared emitted-domain bound, derived from the actual native constructor.
    pub const MAX_ABSOLUTE_SECONDS: i32 = (SECONDS_PER_DAY - 1) as i32;
    pub(crate) fn from_data(seconds: i64) -> Result<Self, InvalidTimeZoneData> {
        if !(-SECONDS_PER_DAY + 1..SECONDS_PER_DAY).contains(&seconds) {
            return Err(InvalidTimeZoneData("named offset is not less than one day"));
        }
        Ok(Self(seconds as i32))
    }
    pub const fn seconds(self) -> i32 {
        self.0
    }
    pub fn encode(self) -> Vec<u8> {
        i64::from(self.0).to_le_bytes().to_vec()
    }
}

fn split(value: i128) -> Result<(i64, u32), InvalidTimeZoneRequest> {
    Ok((
        i64::try_from(value.div_euclid(NANOS_PER_SECOND))
            .map_err(|_| InvalidTimeZoneRequest("time coordinate overflow"))?,
        value.rem_euclid(NANOS_PER_SECOND) as u32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_domains_preserve_negative_fraction_and_endpoint_rules() {
        for (ns, sec, nano) in [
            (-1_000_000_001, -2, 999_999_999),
            (-1_000_000_000, -1, 0),
            (-1, -1, 999_999_999),
            (0, 0, 0),
        ] {
            let instant = TimeZoneInstant::from_nanoseconds(ns).unwrap();
            assert_eq!((instant.seconds(), instant.nanosecond()), (sec, nano));
            assert_eq!(instant.nanoseconds(), ns);
        }
        let limit = i128::from(INSTANT_LIMIT_SECONDS) * NANOS_PER_SECOND;
        assert!(TimeZoneInstant::from_nanoseconds(-limit).is_ok());
        assert!(TimeZoneInstant::from_nanoseconds(limit).is_ok());
        assert!(TimeZoneInstant::from_nanoseconds(limit + 1).is_err());
        assert!(TimeZoneInstant::from_nanoseconds(-limit - 1).is_err());
        assert!(LocalTimeCoordinate::from_nanoseconds(-limit - 1).is_ok());
        assert!(RawTimeZoneEpoch::from_nanoseconds(limit + 1).is_ok());
        assert!(TimeZoneInstant::new(0, 1_000_000_000).is_err());
        assert!(LocalTimeCoordinate::new(i64::MAX, 0).is_err());
        assert!(NamedTimeZoneOffsetSeconds::from_data(86_400).is_err());
    }
}

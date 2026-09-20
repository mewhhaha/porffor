// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

// Adapted from ICU4X 2.1, revision
// 38a49da495248dd1ded84cf306e4ca42e64d5bb3, cal/east_asian_traditional/simple.rs.
// It retains that documented píngqì model while using exact Euclidean i128
// arithmetic and checked calendar boundaries instead of discarded writes.

use calendrical_calculations::{
    chinese_based::ChineseBased, gregorian::fixed_from_gregorian, rata_die::RataDie,
};

const MILLIS_PER_DAY: i128 = 86_400_000;
const MEAN_YEAR_MILLIS: i128 = MILLIS_PER_DAY * 146_097 / 400;
const MEAN_SOLAR_TERM_MILLIS: i128 = MEAN_YEAR_MILLIS / 12;
const MEAN_LUNAR_MONTH_MILLIS: i128 = MILLIS_PER_DAY * 295_305_888_531 / 10_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, displaydoc::Display)]
pub(crate) enum InvalidProlepticYear {
    /// observatory offset is not a finite fraction of one day
    InvalidObservatoryOffset,
    /// proleptic year has an invalid number of months
    MonthCount,
    /// proleptic month length is {0} days
    MonthLength(i64),
    /// proleptic leap month has an invalid ordinal
    LeapOrdinal,
    /// proleptic New Year offset is {0} days after January 1
    NewYearWindow(i64),
    /// proleptic solar terms were not consumed exactly once
    UnconsumedSolarTerm,
    /// proleptic fixed date does not fit i64
    FixedDateRange,
}

#[derive(Debug, Clone, Copy)]
struct LocalMoment(i128);

impl LocalMoment {
    fn at(date: RataDie, millis: i128) -> Self {
        Self(i128::from(date.to_i64_date()) * MILLIS_PER_DAY + millis)
    }

    fn fixed_date(self) -> Result<i64, InvalidProlepticYear> {
        i64::try_from(self.0.div_euclid(MILLIS_PER_DAY))
            .map_err(|_| InvalidProlepticYear::FixedDateRange)
    }

    fn advance(self, millis: i128) -> Self {
        Self(self.0 + millis)
    }

    fn period_on_or_before(date: RataDie, base: Self, period: i128) -> Self {
        let inclusive_end = (i128::from(date.to_i64_date()) + 1) * MILLIS_PER_DAY - 1;
        let periods = (inclusive_end - base.0).div_euclid(period);
        Self(base.0 + periods * period)
    }
}

/// A closed year whose month bounds prove the packed calendar representation.
#[derive(Debug, Clone)]
pub(super) struct ProlepticYear {
    related_iso: i32,
    boundaries: [i64; 14],
    month_count: u8,
    leap_ordinal: Option<u8>,
}

// Construction bounds the month count by 13 before writing the 14 boundaries;
// all field access after construction uses that private validated count.
#[allow(clippy::indexing_slicing)]
impl ProlepticYear {
    pub(super) fn compute<C: ChineseBased>(related_iso: i32) -> Result<Self, InvalidProlepticYear> {
        // The canonical ChineseBased trait owns the historical observatory
        // offset. It is unrelated to Intl's user-selected IANA time zone.
        let offset = C::utc_offset(fixed_from_gregorian(related_iso, 7, 1)) * MILLIS_PER_DAY as f64;
        if !offset.is_finite() || offset.abs() >= MILLIS_PER_DAY as f64 {
            return Err(InvalidProlepticYear::InvalidObservatoryOffset);
        }
        // The trait supplies a rational observatory offset as days. Rounding
        // this bounded value to milliseconds also works in this no_std crate.
        let offset = (offset + if offset < 0.0 { -0.5 } else { 0.5 }) as i128;
        let solstice = LocalMoment::at(
            fixed_from_gregorian(1999, 12, 22),
            ((7 * 60 + 44) * 60 * 1000) + offset,
        );
        let reference_moon = LocalMoment::at(
            fixed_from_gregorian(2000, 1, 6),
            ((18 * 60 + 14) * 60 * 1000) + offset,
        );
        let mut major_term = LocalMoment::period_on_or_before(
            fixed_from_gregorian(related_iso, 1, 1) - 1,
            solstice,
            MEAN_YEAR_MILLIS,
        );
        let mut moon = LocalMoment::period_on_or_before(
            RataDie::new(major_term.fixed_date()?),
            reference_moon,
            MEAN_LUNAR_MONTH_MILLIS,
        );
        let mut next_moon = moon.advance(MEAN_LUNAR_MONTH_MILLIS);
        let mut term = -2;
        let mut leap_in_sui = false;
        let mut preceding_months = 0;
        while term < 0 || (next_moon.fixed_date()? <= major_term.fixed_date()? && !leap_in_sui) {
            preceding_months += 1;
            if preceding_months > 4 {
                return Err(InvalidProlepticYear::UnconsumedSolarTerm);
            }
            if next_moon.fixed_date()? <= major_term.fixed_date()? && !leap_in_sui {
                leap_in_sui = true;
            } else {
                term += 1;
                major_term = major_term.advance(MEAN_SOLAR_TERM_MILLIS);
            }
            (moon, next_moon) = (next_moon, next_moon.advance(MEAN_LUNAR_MONTH_MILLIS));
        }
        if term != 0 {
            return Err(InvalidProlepticYear::UnconsumedSolarTerm);
        }
        let mut boundaries = [0; 14];
        boundaries[0] = moon.fixed_date()?;
        let mut month_count = 0;
        let mut leap_ordinal = None;
        while term < 12 || (next_moon.fixed_date()? <= major_term.fixed_date()? && !leap_in_sui) {
            if month_count >= 13 {
                return Err(InvalidProlepticYear::MonthCount);
            }
            month_count += 1;
            let following = next_moon.fixed_date()?;
            if !matches!(following - boundaries[month_count - 1], 29 | 30) {
                return Err(InvalidProlepticYear::MonthLength(
                    following - boundaries[month_count - 1],
                ));
            }
            boundaries[month_count] = following;
            if following <= major_term.fixed_date()? && !leap_in_sui {
                leap_in_sui = true;
                leap_ordinal = Some(month_count as u8);
            } else {
                term += 1;
                major_term = major_term.advance(MEAN_SOLAR_TERM_MILLIS);
            }
            next_moon = next_moon.advance(MEAN_LUNAR_MONTH_MILLIS);
        }
        if term != 12 {
            return Err(InvalidProlepticYear::UnconsumedSolarTerm);
        }
        if month_count != 12 + usize::from(leap_ordinal.is_some()) {
            return Err(InvalidProlepticYear::MonthCount);
        }
        if leap_ordinal.is_some_and(|ordinal| !(2..=13).contains(&ordinal)) {
            return Err(InvalidProlepticYear::LeapOrdinal);
        }
        // The documented upstream approximation may put New Year on Feb22.
        let january = fixed_from_gregorian(related_iso, 1, 1).to_i64_date();
        if !(18..=52).contains(&(boundaries[0] - january)) {
            return Err(InvalidProlepticYear::NewYearWindow(boundaries[0] - january));
        }
        Ok(Self {
            related_iso,
            boundaries,
            month_count: month_count as u8,
            leap_ordinal,
        })
    }

    pub(super) fn related_iso(&self) -> i32 {
        self.related_iso
    }

    pub(super) fn new_year(&self) -> i64 {
        self.boundaries[0]
    }

    pub(super) fn next_new_year(&self) -> i64 {
        self.boundaries[usize::from(self.month_count)]
    }

    pub(super) fn leap_ordinal(&self) -> Option<u8> {
        self.leap_ordinal
    }

    pub(super) fn month_days(&self, ordinal: u8) -> Option<u8> {
        if !(1..=self.month_count).contains(&ordinal) {
            return None;
        }
        Some(
            (self.boundaries[usize::from(ordinal)] - self.boundaries[usize::from(ordinal - 1)])
                as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalMoment, MILLIS_PER_DAY};
    use calendrical_calculations::rata_die::RataDie;

    #[test]
    fn exact_negative_midnight_stays_on_that_day() {
        for (millis, expected_day) in [
            (-MILLIS_PER_DAY - 1, -2),
            (-MILLIS_PER_DAY, -1),
            (-MILLIS_PER_DAY + 1, -1),
            (-1, -1),
            (0, 0),
            (1, 0),
            (MILLIS_PER_DAY - 1, 0),
            (MILLIS_PER_DAY, 1),
        ] {
            assert_eq!(LocalMoment(millis).fixed_date(), Ok(expected_day));
        }
    }

    #[test]
    fn period_lookup_includes_last_millisecond_but_excludes_next_midnight() {
        for day in [-100_000_001, -2, -1, 0, 1, 100_000_001] {
            let last = LocalMoment::period_on_or_before(
                RataDie::new(day),
                LocalMoment(MILLIS_PER_DAY - 1),
                MILLIS_PER_DAY,
            );
            assert_eq!(last.0, i128::from(day + 1) * MILLIS_PER_DAY - 1);
            let midnight =
                LocalMoment::period_on_or_before(RataDie::new(day), LocalMoment(0), MILLIS_PER_DAY);
            assert_eq!(midnight.0, i128::from(day) * MILLIS_PER_DAY);
        }
    }
}

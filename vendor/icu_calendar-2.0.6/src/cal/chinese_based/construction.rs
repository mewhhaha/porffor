// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use super::{proleptic, ChineseBasedYearInfo};
use crate::provider::chinese_based::{
    ChineseBasedCache, InvalidPackedChineseBasedYear, PackedChineseBasedYearInfo,
};
use calendrical_calculations::{
    chinese_based::{self, ChineseBased, YearBounds},
    iso::fixed_from_iso,
    rata_die::RataDie,
};

// Both retained calendars and the documented integer model have these exact
// New Year boundaries: RD -1334565 and RD 1717776. Keep the whole modern cache
// inside the retained calculation interval; never choose a model after error.
const FIRST_CALCULATED_YEAR: i32 = -3653;
const AFTER_LAST_CALCULATED_YEAR: i32 = 4704;

#[derive(Debug, displaydoc::Display)]
pub(crate) enum InvalidChineseBasedYear {
    /// invalid packed year: {0}
    Packed(InvalidPackedChineseBasedYear),
    /// invalid proleptic year: {0}
    Proleptic(proleptic::InvalidProlepticYear),
    /// astronomical month {ordinal} has {days} days
    MonthLength { ordinal: u8, days: i64 },
    /// astronomical month {ordinal} disagrees with its encoded length
    MonthEncoding { ordinal: u8 },
    /// month intervals do not end at the next New Year
    YearSpan,
    /// related year does not contain ISO July 1
    JulyContainment,
    /// cached year range does not fit i32
    CacheYearRange,
    /// cached New Year does not join the preceding year at {related_iso}
    CacheJoin { related_iso: i32 },
}

impl ChineseBasedYearInfo {
    pub(super) fn from_packed(
        related_iso: i32,
        packed: PackedChineseBasedYearInfo,
    ) -> Result<Self, InvalidChineseBasedYear> {
        let year = Self {
            related_iso,
            packed_data: packed.validate().map_err(InvalidChineseBasedYear::Packed)?,
        };
        let july = fixed_from_iso(related_iso, 7, 1);
        if !(year.new_year() <= july && july < year.next_new_year()) {
            return Err(InvalidChineseBasedYear::JulyContainment);
        }
        Ok(year)
    }

    fn from_months(
        related_iso: i32,
        new_year: RataDie,
        next_new_year: RataDie,
        month_lengths: [bool; 13],
        leap_ordinal: Option<u8>,
    ) -> Result<Self, InvalidChineseBasedYear> {
        let offset = new_year - fixed_from_iso(related_iso, 1, 1);
        let packed = PackedChineseBasedYearInfo::try_new(month_lengths, leap_ordinal, offset)
            .map_err(InvalidChineseBasedYear::Packed)?;
        let year = Self::from_packed(related_iso, packed)?;
        if year.next_new_year() != next_new_year {
            return Err(InvalidChineseBasedYear::YearSpan);
        }
        Ok(year)
    }

    pub(super) fn try_compute<CB: ChineseBased>(
        related_iso: i32,
    ) -> Result<Self, InvalidChineseBasedYear> {
        if !(FIRST_CALCULATED_YEAR..AFTER_LAST_CALCULATED_YEAR).contains(&related_iso) {
            let proleptic = proleptic::ProlepticYear::compute::<CB>(related_iso)
                .map_err(InvalidChineseBasedYear::Proleptic)?;
            let month_lengths =
                core::array::from_fn(|index| proleptic.month_days(index as u8 + 1) == Some(30));
            return Self::from_months(
                proleptic.related_iso(),
                RataDie::new(proleptic.new_year()),
                RataDie::new(proleptic.next_new_year()),
                month_lengths,
                proleptic.leap_ordinal(),
            );
        }

        let YearBounds {
            new_year,
            next_new_year,
        } = YearBounds::compute::<CB>(fixed_from_iso(related_iso, 7, 1));
        let (month_lengths, leap_ordinal) =
            chinese_based::month_structure_for_year::<CB>(new_year, next_new_year);
        let month_count = 12 + u8::from(leap_ordinal.is_some());
        let mut previous = new_year;
        for ordinal in 1..=month_count {
            let (_, next) = chinese_based::days_in_month::<CB>(ordinal, new_year, Some(previous));
            let days = next - previous;
            if !matches!(days, 29 | 30) {
                return Err(InvalidChineseBasedYear::MonthLength { ordinal, days });
            }
            #[allow(clippy::indexing_slicing)]
            // Ordinals 1..=month_count address this 13-month array.
            if month_lengths[usize::from(ordinal - 1)] != (days == 30) {
                return Err(InvalidChineseBasedYear::MonthEncoding { ordinal });
            }
            previous = next;
        }
        if previous != next_new_year {
            return Err(InvalidChineseBasedYear::YearSpan);
        }
        Self::from_months(
            related_iso,
            new_year,
            next_new_year,
            month_lengths,
            leap_ordinal,
        )
    }
}

impl ChineseBasedCache<'_> {
    pub(crate) fn validate<CB: ChineseBased>(&self) -> Result<(), InvalidChineseBasedYear> {
        let mut previous: Option<ChineseBasedYearInfo> = None;
        for (index, packed) in self.data.iter().enumerate() {
            let related_iso = i64::from(self.first_related_iso_year)
                .checked_add(
                    i64::try_from(index).map_err(|_| InvalidChineseBasedYear::CacheYearRange)?,
                )
                .and_then(|year| i32::try_from(year).ok())
                .ok_or(InvalidChineseBasedYear::CacheYearRange)?;
            let year = ChineseBasedYearInfo::from_packed(related_iso, packed)?;
            let before = match previous {
                Some(previous) => previous,
                None => ChineseBasedYearInfo::try_compute::<CB>(
                    related_iso
                        .checked_sub(1)
                        .ok_or(InvalidChineseBasedYear::CacheYearRange)?,
                )?,
            };
            if before.next_new_year() != year.new_year() {
                return Err(InvalidChineseBasedYear::CacheJoin { related_iso });
            }
            previous = Some(year);
        }
        if let Some(last) = previous {
            let related_iso = last
                .related_iso
                .checked_add(1)
                .ok_or(InvalidChineseBasedYear::CacheYearRange)?;
            let following = ChineseBasedYearInfo::try_compute::<CB>(related_iso)?;
            if last.next_new_year() != following.new_year() {
                return Err(InvalidChineseBasedYear::CacheJoin { related_iso });
            }
        }
        Ok(())
    }
}

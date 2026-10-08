//! Fixed little-endian format shared by build-time validation and emission.

pub(crate) const TEMPORAL_EAST_ASIAN_YEAR_ENVELOPE: i64 = 1 << 34;
pub(crate) const EAST_ASIAN_MILLIS_PER_DAY: i64 = 86_400_000;
pub(crate) const EAST_ASIAN_MEAN_YEAR_MILLIS: i64 =
    (EAST_ASIAN_MILLIS_PER_DAY as i128 * 146_097 / 400) as i64;
pub(crate) const EAST_ASIAN_MEAN_SOLAR_TERM_MILLIS: i64 = EAST_ASIAN_MEAN_YEAR_MILLIS / 12;
pub(crate) const EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS: i64 =
    (EAST_ASIAN_MILLIS_PER_DAY as i128 * 295_305_888_531 / 10_000_000_000) as i64;

pub(super) const FIRST_RELATED_YEAR: i32 = -3653;
pub(super) const AFTER_LAST_RELATED_YEAR: i32 = 4704;
pub(super) const RETAINED_YEAR_COUNT: usize =
    (AFTER_LAST_RELATED_YEAR - FIRST_RELATED_YEAR) as usize;

/// The sole native kind also consumed by calendar admission and the year model.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalEastAsianCalendar {
    Chinese,
    Dangi,
}

pub(crate) const EAST_ASIAN_YEAR_ROW_BYTES: u64 = 8;
pub(crate) const EAST_ASIAN_MONTH_MASK: u32 = (1 << 13) - 1;
pub(crate) const EAST_ASIAN_LEAP_ORDINAL_SHIFT: u32 = 13;
pub(crate) const EAST_ASIAN_NEW_YEAR_OFFSET_SHIFT: u32 = 17;

/// Both words use a four-byte load; MonthPrefix is extended as signed i32.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalEastAsianYearRowSlot {
    Packed,
    MonthPrefix,
}

impl TemporalEastAsianYearRowSlot {
    pub(crate) const fn byte_offset(self) -> u64 {
        match self {
            Self::Packed => 0,
            Self::MonthPrefix => 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ModelAnchors {
    pub(super) lower_reference_moon_millis: i64,
    pub(super) upper_reference_moon_millis: i64,
    pub(super) lower_solstice_millis: i64,
    pub(super) upper_solstice_millis: i64,
}

pub(super) const HEADER_BYTES: usize = 7 * 8;
pub(super) const CALENDAR_IMAGE_BYTES: usize =
    HEADER_BYTES + RETAINED_YEAR_COUNT * EAST_ASIAN_YEAR_ROW_BYTES as usize;
pub(super) const IMAGE_BYTES: usize = CALENDAR_IMAGE_BYTES * 2;

#[derive(Clone, Copy, Debug)]
pub(super) struct CalendarHeader {
    pub(super) total_months: i64,
    pub(super) lower_serial_offset: i64,
    pub(super) upper_serial_offset: i64,
    pub(super) anchors: ModelAnchors,
}

impl CalendarHeader {
    pub(super) fn from_le_bytes(bytes: &[u8; HEADER_BYTES]) -> Self {
        let mut words = [0; 7];
        for (word, slot) in words.iter_mut().zip(bytes.chunks_exact(8)) {
            *word = i64::from_le_bytes(slot.try_into().expect("one eight-byte word"));
        }
        let [total_months, lower_serial_offset, upper_serial_offset, lower_reference_moon_millis, upper_reference_moon_millis, lower_solstice_millis, upper_solstice_millis] =
            words;
        Self {
            total_months,
            lower_serial_offset,
            upper_serial_offset,
            anchors: ModelAnchors {
                lower_reference_moon_millis,
                upper_reference_moon_millis,
                lower_solstice_millis,
                upper_solstice_millis,
            },
        }
    }
}

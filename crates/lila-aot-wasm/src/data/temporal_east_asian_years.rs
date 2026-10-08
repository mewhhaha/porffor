//! Checked Chinese/Dangi years, validated once while building the compiler.
//!
//! Each worker copies the immutable retained rows instead of reconstructing
//! 16,714 provider years. Emitted helpers and their Wasm data are unchanged.

use super::{EmitError, StringPool, STATIC_DATA_OFFSET};
#[cfg(test)]
mod construction;
mod format;
use format::{
    CalendarHeader, CALENDAR_IMAGE_BYTES, FIRST_RELATED_YEAR, HEADER_BYTES, IMAGE_BYTES,
    RETAINED_YEAR_COUNT,
};
pub(crate) use format::{
    TemporalEastAsianCalendar, TemporalEastAsianYearRowSlot, EAST_ASIAN_LEAP_ORDINAL_SHIFT,
    EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS, EAST_ASIAN_MEAN_SOLAR_TERM_MILLIS,
    EAST_ASIAN_MEAN_YEAR_MILLIS, EAST_ASIAN_MILLIS_PER_DAY, EAST_ASIAN_MONTH_MASK,
    EAST_ASIAN_NEW_YEAR_OFFSET_SHIFT, EAST_ASIAN_YEAR_ROW_BYTES, TEMPORAL_EAST_ASIAN_YEAR_ENVELOPE,
};

// Cargo rebuilds this image whenever its constructor, format or pinned provider
// changes. A malformed length cannot compile into the product.
const EMBEDDED_IMAGE: &[u8; IMAGE_BYTES] =
    include_bytes!(concat!(env!("OUT_DIR"), "/temporal-east-asian-years.bin"));

fn image_range() -> EmitError {
    EmitError::unsupported("Temporal Chinese/Dangi image exceeds Wasm memory32")
}

/// Checked addresses and native header values of one retained-year image.
#[derive(Debug)]
pub(crate) struct TemporalEastAsianCalendarImage {
    rows_ptr: u32,
    header: CalendarHeader,
}

impl TemporalEastAsianCalendarImage {
    pub(crate) const fn rows_ptr(&self) -> u32 {
        self.rows_ptr
    }
    pub(crate) const fn first_related_year(&self) -> i64 {
        FIRST_RELATED_YEAR as i64
    }
    pub(crate) const fn year_count(&self) -> u32 {
        RETAINED_YEAR_COUNT as u32
    }
    pub(crate) const fn total_months(&self) -> i64 {
        self.header.total_months
    }
    pub(crate) const fn lower_serial_offset(&self) -> i64 {
        self.header.lower_serial_offset
    }
    pub(crate) const fn upper_serial_offset(&self) -> i64 {
        self.header.upper_serial_offset
    }
    pub(crate) const fn lower_reference_moon_millis(&self) -> i64 {
        self.header.anchors.lower_reference_moon_millis
    }
    pub(crate) const fn upper_reference_moon_millis(&self) -> i64 {
        self.header.anchors.upper_reference_moon_millis
    }
    pub(crate) const fn lower_solstice_millis(&self) -> i64 {
        self.header.anchors.lower_solstice_millis
    }
    pub(crate) const fn upper_solstice_millis(&self) -> i64 {
        self.header.anchors.upper_solstice_millis
    }

    fn append(bytes: &mut Vec<u8>, image: &[u8; CALENDAR_IMAGE_BYTES]) -> Result<Self, EmitError> {
        let rows_ptr = STATIC_DATA_OFFSET
            .checked_add(u32::try_from(bytes.len()).map_err(|_| image_range())?)
            .ok_or_else(image_range)?;
        let (header, rows) = image
            .split_first_chunk::<HEADER_BYTES>()
            .expect("build-time calendar image has a complete fixed header");
        bytes.extend_from_slice(rows);
        Ok(Self {
            rows_ptr,
            header: CalendarHeader::from_le_bytes(header),
        })
    }
}

/// Named images make omission of either kind visible in exhaustive dispatch.
#[derive(Debug)]
pub(crate) struct TemporalEastAsianYearImage {
    chinese: TemporalEastAsianCalendarImage,
    dangi: TemporalEastAsianCalendarImage,
}

impl TemporalEastAsianYearImage {
    pub(crate) fn calendar(
        &self,
        calendar: TemporalEastAsianCalendar,
    ) -> &TemporalEastAsianCalendarImage {
        match calendar {
            TemporalEastAsianCalendar::Chinese => &self.chinese,
            TemporalEastAsianCalendar::Dangi => &self.dangi,
        }
    }

    fn append_to(bytes: &mut Vec<u8>) -> Result<Self, EmitError> {
        let alignment = EAST_ASIAN_YEAR_ROW_BYTES as usize;
        let padding = (alignment - bytes.len() % alignment) % alignment;
        let aligned = bytes.len().checked_add(padding).ok_or_else(image_range)?;
        let row_bytes = RETAINED_YEAR_COUNT
            .checked_mul(EAST_ASIAN_YEAR_ROW_BYTES as usize)
            .and_then(|one| one.checked_mul(2))
            .ok_or_else(image_range)?;
        let end = aligned.checked_add(row_bytes).ok_or_else(image_range)?;
        STATIC_DATA_OFFSET
            .checked_add(u32::try_from(end).map_err(|_| image_range())?)
            .ok_or_else(image_range)?;
        bytes.try_reserve_exact(end - bytes.len()).map_err(|_| {
            EmitError::unsupported("Temporal Chinese/Dangi image allocation failed")
        })?;
        bytes.resize(aligned, 0);
        let (chinese, dangi) = EMBEDDED_IMAGE
            .split_first_chunk::<CALENDAR_IMAGE_BYTES>()
            .expect("build-time image contains two complete calendars");
        Ok(Self {
            chinese: TemporalEastAsianCalendarImage::append(bytes, chinese)?,
            dangi: TemporalEastAsianCalendarImage::append(
                bytes,
                dangi
                    .try_into()
                    .expect("build-time image contains one complete Dangi calendar"),
            )?,
        })
    }
}

impl StringPool {
    pub(super) fn append_temporal_east_asian_year_image(&mut self) -> Result<(), EmitError> {
        self.temporal_east_asian_year_image =
            Some(TemporalEastAsianYearImage::append_to(&mut self.bytes)?);
        Ok(())
    }

    pub(crate) fn temporal_east_asian_year_image(&self) -> &TemporalEastAsianYearImage {
        self.temporal_east_asian_year_image
            .as_ref()
            .expect("Temporal calendar compiler requires its checked year image")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_east_asian_catalog_matches_every_checked_provider_row() {
        let reference = construction::build_image().expect("complete pinned Date catalog");
        assert_eq!(reference.len(), EMBEDDED_IMAGE.len());
        for (offset, (expected, actual)) in reference.iter().zip(EMBEDDED_IMAGE).enumerate() {
            assert_eq!(actual, expected, "catalog byte {offset}");
        }
        // Exercise the actual pool append at every possible starting alignment.
        for prefix in 0..EAST_ASIAN_YEAR_ROW_BYTES as usize {
            let mut bytes = vec![0xA5; prefix];
            let image = TemporalEastAsianYearImage::append_to(&mut bytes).unwrap();
            let start = if prefix == 0 {
                0
            } else {
                EAST_ASIAN_YEAR_ROW_BYTES as usize
            };
            let rows = RETAINED_YEAR_COUNT * EAST_ASIAN_YEAR_ROW_BYTES as usize;
            assert_eq!(&bytes[..prefix], vec![0xA5; prefix]);
            assert!(bytes[prefix..start].iter().all(|byte| *byte == 0));
            for (ordinal, kind) in [
                TemporalEastAsianCalendar::Chinese,
                TemporalEastAsianCalendar::Dangi,
            ]
            .into_iter()
            .enumerate()
            {
                let calendar = image.calendar(kind);
                let offset = start + ordinal * rows;
                let reference_start = ordinal * CALENDAR_IMAGE_BYTES;
                assert_eq!(
                    calendar.rows_ptr() as usize,
                    STATIC_DATA_OFFSET as usize + offset
                );
                assert_eq!(
                    &bytes[offset..offset + rows],
                    &reference
                        [reference_start + HEADER_BYTES..reference_start + CALENDAR_IMAGE_BYTES]
                );
                assert_eq!(
                    &calendar.header.to_le_bytes(),
                    &reference[reference_start..reference_start + HEADER_BYTES]
                );
            }
            assert_eq!(bytes.len(), start + 2 * rows);
        }
    }
}

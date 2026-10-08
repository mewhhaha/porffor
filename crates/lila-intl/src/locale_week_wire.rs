use crate::LocaleWeekInfo;

pub const LOCALE_WEEK_WIRE_BYTES: usize = 8;

/// Only checked native week data can create an ABI response record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocaleWeekWire {
    first_day: u32,
    weekend_mask: u32,
}

impl LocaleWeekWire {
    #[must_use]
    pub fn from_info(info: &LocaleWeekInfo) -> Self {
        Self {
            first_day: u32::from(info.first_day().iso_number()),
            weekend_mask: u32::from(info.weekend_mask()),
        }
    }

    #[must_use]
    pub fn encode(self) -> [u8; LOCALE_WEEK_WIRE_BYTES] {
        let mut bytes = [0; LOCALE_WEEK_WIRE_BYTES];
        bytes[..4].copy_from_slice(&self.first_day.to_le_bytes());
        bytes[4..].copy_from_slice(&self.weekend_mask.to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_wire_keeps_two_distinct_little_endian_words() {
        let monday = LocaleWeekInfo::from_iso_mask(1, 0x60).unwrap();
        let friday = LocaleWeekInfo::from_iso_mask(5, 0x01).unwrap();
        assert_eq!(
            LocaleWeekWire::from_info(&monday).encode(),
            [1, 0, 0, 0, 0x60, 0, 0, 0]
        );
        assert_eq!(
            LocaleWeekWire::from_info(&friday).encode(),
            [5, 0, 0, 0, 0x01, 0, 0, 0]
        );
    }
}

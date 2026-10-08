use crate::{DateTimeHourCycle, LocaleHourCycles};

pub const LOCALE_HOUR_CYCLES_WIRE_BYTES: usize = 8;

/// A validated native list is the only way to create an ordered ABI record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocaleHourCyclesWire {
    count: u32,
    packed_cycles: u32,
}

impl LocaleHourCyclesWire {
    #[must_use]
    pub fn from_info(info: &LocaleHourCycles) -> Self {
        let mut packed_cycles = 0;
        for (index, cycle) in info.cycles().iter().enumerate() {
            let code: u32 = match cycle {
                DateTimeHourCycle::H11 => 1,
                DateTimeHourCycle::H12 => 2,
                DateTimeHourCycle::H23 => 3,
                DateTimeHourCycle::H24 => 4,
            };
            packed_cycles |= code << (index * 8);
        }
        Self {
            count: info.cycles().len() as u32,
            packed_cycles,
        }
    }

    #[must_use]
    pub fn encode(self) -> [u8; LOCALE_HOUR_CYCLES_WIRE_BYTES] {
        let mut bytes = [0; LOCALE_HOUR_CYCLES_WIRE_BYTES];
        bytes[..4].copy_from_slice(&self.count.to_le_bytes());
        bytes[4..].copy_from_slice(&self.packed_cycles.to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_cycle_bytes_preserve_preferences_and_zero_unused_slots() {
        use crate::{
            CanonicalLocaleId, EmbeddedIntlProvider, IntlOperationProvider,
            LocaleHourCyclesOperation, LocaleHourCyclesRequest,
        };
        let provider = EmbeddedIntlProvider::new().unwrap();
        for (tag, expected) in [
            ("en-US", [2, 0, 0, 0, 2, 3, 0, 0]),
            ("en-JP", [3, 0, 0, 0, 3, 1, 2, 0]),
            ("en-GB", [2, 0, 0, 0, 3, 2, 0, 0]),
            ("en-AX", [1, 0, 0, 0, 3, 0, 0, 0]),
        ] {
            let request =
                LocaleHourCyclesRequest::new(CanonicalLocaleId::from_data(tag).unwrap()).unwrap();
            let info = <EmbeddedIntlProvider as IntlOperationProvider<LocaleHourCyclesOperation>>::execute(&provider, request).unwrap();
            assert_eq!(LocaleHourCyclesWire::from_info(&info).encode(), expected);
        }
    }
}

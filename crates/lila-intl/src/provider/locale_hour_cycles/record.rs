use crate::DateTimeHourCycle;

use super::LocaleHourCyclesProfileError;

/// A nonempty list whose closed cycle values are unique in preference order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleHourCycles {
    cycles: Box<[DateTimeHourCycle]>,
}

impl LocaleHourCycles {
    pub(super) fn checked(
        cycles: Vec<DateTimeHourCycle>,
    ) -> Result<Self, LocaleHourCyclesProfileError> {
        if cycles.is_empty() || cycles.len() > 4 {
            return Err(LocaleHourCyclesProfileError::Cycles);
        }
        for (index, cycle) in cycles.iter().enumerate() {
            if cycles[..index].contains(cycle) {
                return Err(LocaleHourCyclesProfileError::Cycles);
            }
        }
        Ok(Self {
            cycles: cycles.into_boxed_slice(),
        })
    }

    pub(super) fn single(cycle: DateTimeHourCycle) -> Self {
        Self {
            cycles: Box::new([cycle]),
        }
    }

    #[must_use]
    pub fn cycles(&self) -> &[DateTimeHourCycle] {
        &self.cycles
    }
}

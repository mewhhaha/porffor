use super::*;
pub const DURATION_CONFIGURATION_WORDS: usize = 12;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationConfigurationWord {
    Style,
    FractionalDigits,
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}
impl DurationConfigurationWord {
    pub const ALL: [Self; DURATION_CONFIGURATION_WORDS] = [
        Self::Style,
        Self::FractionalDigits,
        Self::Years,
        Self::Months,
        Self::Weeks,
        Self::Days,
        Self::Hours,
        Self::Minutes,
        Self::Seconds,
        Self::Milliseconds,
        Self::Microseconds,
        Self::Nanoseconds,
    ];
    pub const fn index(self) -> usize {
        match self {
            Self::Style => 0,
            Self::FractionalDigits => 1,
            Self::Years => 2,
            Self::Months => 3,
            Self::Weeks => 4,
            Self::Days => 5,
            Self::Hours => 6,
            Self::Minutes => 7,
            Self::Seconds => 8,
            Self::Milliseconds => 9,
            Self::Microseconds => 10,
            Self::Nanoseconds => 11,
        }
    }
    pub const fn offset(self) -> u64 {
        self.index() as u64 * 8
    }
}
impl CheckedDurationConfiguration {
    pub fn wire_words(&self) -> [u64; DURATION_CONFIGURATION_WORDS] {
        let mut words = [0; DURATION_CONFIGURATION_WORDS];
        words[0] = self.style().index() as u64;
        words[1] = self
            .fractional_digits()
            .map_or(0, |digits| u64::from(digits.value()) + 1);
        for &unit in DurationUnit::ALL {
            let (style, display) = self.unit_options(unit);
            words[unit.index() + 2] = style.index() as u64 | ((display.index() as u64) << 8);
        }
        words
    }
}
pub(super) fn options(
    words: [u64; DURATION_CONFIGURATION_WORDS],
) -> Result<DurationOptions, DurationWireError> {
    let style = match words[0] {
        0 => DurationStyle::Long,
        1 => DurationStyle::Short,
        2 => DurationStyle::Narrow,
        3 => DurationStyle::Digital,
        _ => return Err(DurationWireError::Malformed("duration style")),
    };
    let fractional_digits = match words[1] {
        0 => None,
        1..=10 => Some(DurationFractionalDigits::new((words[1] - 1) as u8)?),
        _ => return Err(DurationWireError::Malformed("fractional digits")),
    };
    let mut units = [DurationUnitOption::default(); 10];
    for &unit in DurationUnit::ALL {
        let word = words[unit.index() + 2];
        if word & !0x1ff != 0 {
            return Err(DurationWireError::Malformed("unit option reserved bits"));
        }
        let style = match word & 0xff {
            0 => DurationUnitStyle::Long,
            1 => DurationUnitStyle::Short,
            2 => DurationUnitStyle::Narrow,
            3 => DurationUnitStyle::Numeric,
            4 => DurationUnitStyle::TwoDigit,
            _ => return Err(DurationWireError::Malformed("unit style")),
        };
        let display = if word & 0x100 == 0 {
            DurationDisplay::Auto
        } else {
            DurationDisplay::Always
        };
        units[unit.index()] = DurationUnitOption {
            style: Some(style),
            display: Some(display),
        };
    }
    Ok(DurationOptions {
        style,
        units,
        fractional_digits,
    })
}

use icu_calendar::types::{MonthCode as NativeMonthCode, MonthInfo};

use super::kind::CalendarId;
use crate::datetime::DateTimeFormatError;

/// Exact checked code; a formatted leap label is separate from its ordinal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::provider::datetime) struct MonthCode {
    native: NativeMonthCode,
    number: u8,
    leap: bool,
}

impl MonthCode {
    fn from_native(native: NativeMonthCode) -> Result<Self, DateTimeFormatError> {
        let (digits, leap) = match native.0.as_str().as_bytes() {
            [b'M', a, b] => ([*a, *b], false),
            [b'M', a, b, b'L'] => ([*a, *b], true),
            _ => return Err(invalid()),
        };
        if !digits.iter().all(u8::is_ascii_digit) {
            return Err(invalid());
        }
        let number = (digits[0] - b'0') * 10 + digits[1] - b'0';
        if !(1..=13).contains(&number) || (number == 13 && leap) {
            return Err(invalid());
        }
        Ok(Self {
            native,
            number,
            leap,
        })
    }

    #[cfg(test)]
    pub(in crate::provider::datetime) const fn native(self) -> NativeMonthCode {
        self.native
    }
    pub(in crate::provider::datetime) const fn number(self) -> u8 {
        self.number
    }
    pub(in crate::provider::datetime) const fn is_leap(self) -> bool {
        self.leap
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::provider::datetime) struct Month {
    ordinal: u8,
    standard: MonthCode,
    formatting: MonthCode,
}

impl Month {
    pub(in crate::provider::datetime) fn from_native(
        calendar: CalendarId,
        native: MonthInfo,
        count: u8,
    ) -> Result<Self, DateTimeFormatError> {
        use CalendarId as C;
        let standard = MonthCode::from_native(native.standard_code)?;
        let formatting = MonthCode::from_native(native.formatting_code)?;
        let valid = match calendar {
            C::Chinese | C::Dangi => {
                (12..=13).contains(&count)
                    && standard.number <= 12
                    && standard == formatting
                    && if standard.leap {
                        count == 13 && native.ordinal == standard.number + 1
                    } else {
                        native.ordinal == standard.number
                            || (count == 13 && native.ordinal == standard.number + 1)
                    }
            }
            C::Hebrew => {
                (12..=13).contains(&count)
                    && standard.number <= 12
                    && match (
                        standard.number,
                        standard.leap,
                        formatting.number,
                        formatting.leap,
                    ) {
                        (5, true, 5, true) => count == 13 && native.ordinal == 6,
                        (6, false, 6, true) => count == 13 && native.ordinal == 7,
                        (6, false, 6, false) => count == 12 && native.ordinal == 6,
                        (_, false, _, false) => {
                            standard == formatting
                                && native.ordinal
                                    == standard.number
                                        + u8::from(count == 13 && standard.number > 6)
                        }
                        _ => false,
                    }
            }
            C::Coptic | C::Ethioaa | C::Ethiopic => {
                count == 13
                    && !standard.leap
                    && standard == formatting
                    && native.ordinal == standard.number
            }
            C::Buddhist
            | C::Gregory
            | C::Indian
            | C::IslamicCivil
            | C::IslamicTbla
            | C::IslamicUmalqura
            | C::Iso8601
            | C::Japanese
            | C::Persian
            | C::Roc => {
                count == 12
                    && standard.number <= 12
                    && !standard.leap
                    && standard == formatting
                    && native.ordinal == standard.number
            }
        };
        if !valid || !(1..=count).contains(&native.ordinal) {
            return Err(invalid());
        }
        Ok(Self {
            ordinal: native.ordinal,
            standard,
            formatting,
        })
    }

    #[cfg(test)]
    pub(in crate::provider::datetime) const fn ordinal(self) -> u8 {
        self.ordinal
    }
    #[cfg(test)]
    pub(in crate::provider::datetime) const fn standard(self) -> MonthCode {
        self.standard
    }
    pub(in crate::provider::datetime) const fn formatting(self) -> MonthCode {
        self.formatting
    }
}

fn invalid() -> DateTimeFormatError {
    DateTimeFormatError::InvalidProfile("native calendar month outside its checked domain".into())
}

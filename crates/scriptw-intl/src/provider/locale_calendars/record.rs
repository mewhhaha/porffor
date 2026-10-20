use super::LocaleCalendarsProfileError;
use crate::DateTimeCalendar;
/// Only checked, consumed-provider calendar names can reach the host response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleCalendars {
    names: Box<[Box<str>]>,
}
impl LocaleCalendars {
    pub(super) fn checked(
        calendars: Vec<DateTimeCalendar>,
    ) -> Result<Self, LocaleCalendarsProfileError> {
        if calendars.is_empty() || calendars.len() > DateTimeCalendar::ALL.len() {
            return Err(LocaleCalendarsProfileError::Calendars);
        }
        for (i, calendar) in calendars.iter().enumerate() {
            if calendars[..i].contains(calendar) {
                return Err(LocaleCalendarsProfileError::Calendars);
            }
        }
        Ok(Self {
            names: calendars
                .into_iter()
                .map(|calendar| Box::<str>::from(calendar.as_str()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
    }
    #[must_use]
    pub fn names(&self) -> &[Box<str>] {
        &self.names
    }
}

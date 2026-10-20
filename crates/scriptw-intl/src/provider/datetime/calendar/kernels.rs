//! One complete admitted calculation owner for the closed DateTime calendars.

use icu_calendar::AnyCalendar;
use icu_provider_blob::BlobDataProvider;
use std::sync::Arc;

use super::CalendarId;
use crate::datetime::{DateTimeCalendar, DateTimeFormatError};

pub(in crate::provider::datetime) struct CalendarKernels {
    calendars: [Arc<AnyCalendar>; 16],
}

impl CalendarKernels {
    pub(in crate::provider::datetime) fn from_data(
        data: &BlobDataProvider,
    ) -> Result<Self, DateTimeFormatError> {
        let calendars = CalendarId::ALL
            .into_iter()
            .map(|kind| {
                AnyCalendar::try_new_with_buffer_provider(data, kind.kind())
                    .map(Arc::new)
                    .map_err(|error| DateTimeFormatError::InvalidProfile(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| {
                DateTimeFormatError::InvalidProfile("calendar inventory changed".into())
            })?;
        Ok(Self { calendars })
    }

    pub(in crate::provider::datetime) fn get(&self, kind: CalendarId) -> Arc<AnyCalendar> {
        Arc::clone(&self.calendars[kind.index()])
    }
    pub(in crate::provider::datetime) fn available_calendars(&self) -> Vec<DateTimeCalendar> {
        DateTimeCalendar::ALL
            .iter()
            .copied()
            .filter(|calendar| {
                self.calendars
                    .get(CalendarId::from_admitted(*calendar).index())
                    .is_some()
            })
            .collect()
    }
}

#[cfg(test)]
pub(in crate::provider::datetime) fn pinned() -> &'static CalendarKernels {
    static KERNELS: std::sync::OnceLock<CalendarKernels> = std::sync::OnceLock::new();
    KERNELS.get_or_init(|| {
        let data =
            BlobDataProvider::try_new_from_static_blob(crate::datetime_image::PINNED_CALENDAR)
                .expect("locked calendar blob");
        CalendarKernels::from_data(&data).expect("complete admitted calendar kernels")
    })
}

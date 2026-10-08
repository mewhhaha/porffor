//! Native Locale information consumes checked tables and selected foundations.

use crate::image::{DataImageComponent, DataImageEnvelope};
use crate::provider::locale_calendars::{self, LocaleCalendarsProfile};
use crate::provider::locale_hour_cycles::{self, LocaleHourCyclesProfile};
use crate::provider::locale_text::{self, LocaleTextProfile};
use crate::provider::locale_week::{self, LocaleWeekProfile};
use crate::provider::{DateTimeProvider, LocaleCanonicalizationData};
use crate::{
    DateTimeDataImage, IntlDataDigest, IntlDataImageError, IntlDataProfile, LocaleCalendars,
    LocaleCalendarsError, LocaleCalendarsRequest, LocaleDataImage, LocaleHourCycles,
    LocaleHourCyclesError, LocaleHourCyclesRequest, LocaleTextDirection, LocaleTextError,
    LocaleTextInfoRequest, LocaleWeekError, LocaleWeekInfo, LocaleWeekRequest,
};
use std::sync::{Arc, OnceLock};

pub const INTL_NATIVE_LOCALE_INFORMATION_CUSTOM_SECTION: &str = "lila.intl-locale-information.v1";
const MARKERS: [&str; 4] = [
    "lila/locale/calendars/profiles/v1",
    "lila/locale/hour-cycles/profiles/v1",
    "lila/locale/text/profiles/v1",
    "lila/locale/week/profiles/v1",
];
const MAGIC: &[u8; 8] = b"LLNI0001";
const CALENDARS: &[u8] = include_bytes!("../data/locale-calendars-cldr-47/profile.json");
const HOUR_CYCLES: &[u8] = include_bytes!("../data/locale-hour-cycles-cldr-47/profile.json");
const TEXT: &[u8] = include_bytes!("../data/locale-text-cldr-47/profile.json");
const WEEK: &[u8] = include_bytes!("../data/locale-week-cldr-47/profile.json");
const TABLES: [&[u8]; 4] = [CALENDARS, HOUR_CYCLES, TEXT, WEEK];

fn native_payload() -> Result<Vec<u8>, IntlDataImageError> {
    let size = TABLES.iter().try_fold(8usize, |size, table| {
        size.checked_add(8)
            .and_then(|size| size.checked_add(table.len()))
            .ok_or_else(|| IntlDataImageError::consumer("native Locale information extent"))
    })?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(IntlDataImageError::consumer)?;
    bytes.extend_from_slice(MAGIC);
    for table in TABLES {
        bytes.extend_from_slice(
            &u64::try_from(table.len())
                .map_err(IntlDataImageError::consumer)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(table);
    }
    Ok(bytes)
}

fn admit_tables(bytes: &[u8]) -> Result<[&[u8]; 4], IntlDataImageError> {
    let fail = || IntlDataImageError::consumer("native Locale information framing or locked data");
    if bytes.get(..8) != Some(MAGIC.as_slice()) {
        return Err(fail());
    }
    let mut position = 8usize;
    let mut tables = [&[][..]; 4];
    for (table, pinned) in tables.iter_mut().zip(TABLES) {
        let end = position.checked_add(8).ok_or_else(fail)?;
        let size = bytes.get(position..end).ok_or_else(fail)?;
        let size = usize::try_from(u64::from_le_bytes(
            size.try_into().expect("eight-byte extent"),
        ))
        .map_err(IntlDataImageError::consumer)?;
        position = end;
        let end = position.checked_add(size).ok_or_else(fail)?;
        *table = bytes.get(position..end).ok_or_else(fail)?;
        if *table != pinned {
            return Err(fail());
        }
        position = end;
    }
    if position != bytes.len() {
        return Err(fail());
    }
    Ok(tables)
}

struct AdmittedNativeLocaleInformation {
    envelope: DataImageEnvelope,
    calendars: LocaleCalendarsProfile,
    hour_cycles: LocaleHourCyclesProfile,
    text: LocaleTextProfile,
    week: LocaleWeekProfile,
    locale: LocaleDataImage,
    authority: LocaleCanonicalizationData,
    date_time: Arc<DateTimeProvider>,
    available_calendars: Box<[crate::DateTimeCalendar]>,
    date_time_digest: IntlDataDigest,
}

/// Only checked constructors can publish all four tables. Calendar results use
/// the actual selected DateTime provider's full admitted calculation kernels,
/// independently of its selected localized name/pattern associations.
#[derive(Clone)]
pub struct NativeLocaleInformationDataImage(Arc<AdmittedNativeLocaleInformation>);
impl core::fmt::Debug for NativeLocaleInformationDataImage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeLocaleInformationDataImage")
            .field("digest", &self.digest())
            .field("locale_digest", &self.locale_digest())
            .field("date_time_digest", &self.date_time_digest())
            .finish()
    }
}
impl NativeLocaleInformationDataImage {
    pub fn for_profile(
        profile: IntlDataProfile,
        locale: &LocaleDataImage,
        date_time: &DateTimeDataImage,
    ) -> Result<Self, IntlDataImageError> {
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::NativeLocaleInformation,
                &profile,
                &MARKERS,
                &native_payload()?,
            )?,
            locale,
            date_time,
        )
    }
    pub fn from_bytes(
        bytes: Arc<[u8]>,
        locale: &LocaleDataImage,
        date_time: &DateTimeDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let envelope = DataImageEnvelope::decode(
            bytes,
            DataImageComponent::NativeLocaleInformation,
            &MARKERS,
        )?;
        if envelope.profile() != locale.profile()
            || envelope.profile() != date_time.profile()
            || !date_time.uses_locale(locale)
        {
            return Err(IntlDataImageError::consumer(
                "native Locale information foundations differ",
            ));
        }
        let [calendars, hour_cycles, text, week] = admit_tables(envelope.blob())?;
        let calendars = LocaleCalendarsProfile::from_bytes(
            calendars,
            locale_calendars::LOCALE_CALENDARS_DATA_SHA256,
        )
        .map_err(IntlDataImageError::consumer)?;
        let hour_cycles = LocaleHourCyclesProfile::from_bytes(
            hour_cycles,
            locale_hour_cycles::LOCALE_HOUR_CYCLES_DATA_SHA256,
        )
        .map_err(IntlDataImageError::consumer)?;
        let text = LocaleTextProfile::from_bytes(text, locale_text::LOCALE_TEXT_DATA_SHA256)
            .map_err(|error| {
                IntlDataImageError::consumer(format!("Locale text table: {error:?}"))
            })?;
        let week = LocaleWeekProfile::from_bytes(week, locale_week::LOCALE_WEEK_DATA_SHA256)
            .map_err(|error| {
                IntlDataImageError::consumer(format!("Locale week table: {error:?}"))
            })?;
        let authority =
            LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
        let provider = date_time.provider();
        let available = provider.available_calendar_kernels();
        if !available.contains(&crate::DateTimeCalendar::Gregorian) {
            return Err(IntlDataImageError::consumer(
                "selected DateTime Gregorian fallback is unavailable",
            ));
        }
        Ok(Self(Arc::new(AdmittedNativeLocaleInformation {
            envelope,
            calendars,
            hour_cycles,
            text,
            week,
            locale: locale.clone(),
            authority,
            date_time: provider,
            available_calendars: available.into_boxed_slice(),
            date_time_digest: date_time.digest(),
        })))
    }
    pub fn bytes(&self) -> Arc<[u8]> {
        self.0.envelope.bytes()
    }
    pub fn digest(&self) -> IntlDataDigest {
        self.0.envelope.digest()
    }
    pub fn profile(&self) -> &IntlDataProfile {
        self.0.envelope.profile()
    }
    pub(crate) fn locale_digest(&self) -> IntlDataDigest {
        self.0.locale.digest()
    }
    pub(crate) fn date_time_digest(&self) -> IntlDataDigest {
        self.0.date_time_digest
    }
    pub(crate) fn uses_foundations(
        &self,
        locale: &LocaleDataImage,
        date_time: &Arc<DateTimeProvider>,
    ) -> bool {
        self.0.locale.same_owner(locale) && Arc::ptr_eq(&self.0.date_time, date_time)
    }
    pub(crate) fn resolve_calendars(
        &self,
        request: LocaleCalendarsRequest,
    ) -> Result<LocaleCalendars, LocaleCalendarsError> {
        locale_calendars::resolve_locale_calendars(
            request,
            &self.0.calendars,
            &self.0.authority,
            &self.0.available_calendars,
        )
    }
    pub(crate) fn resolve_hour_cycles(
        &self,
        request: LocaleHourCyclesRequest,
    ) -> Result<LocaleHourCycles, LocaleHourCyclesError> {
        locale_hour_cycles::resolve_locale_hour_cycles(
            request,
            &self.0.hour_cycles,
            &self.0.authority,
        )
    }
    pub(crate) fn resolve_text(
        &self,
        request: LocaleTextInfoRequest,
    ) -> Result<Option<LocaleTextDirection>, LocaleTextError> {
        locale_text::resolve_locale_text(request, &self.0.text, &self.0.authority)
    }
    pub(crate) fn resolve_week(
        &self,
        request: LocaleWeekRequest,
    ) -> Result<LocaleWeekInfo, LocaleWeekError> {
        locale_week::resolve_locale_week(request, &self.0.week, &self.0.authority)
    }
    #[cfg(test)]
    pub(crate) fn calendars_profile(&self) -> &LocaleCalendarsProfile {
        &self.0.calendars
    }
    #[cfg(test)]
    pub(crate) fn hour_cycles_profile(&self) -> &LocaleHourCyclesProfile {
        &self.0.hour_cycles
    }
    #[cfg(test)]
    pub(crate) fn text_profile(&self) -> &LocaleTextProfile {
        &self.0.text
    }
    #[cfg(test)]
    pub(crate) fn week_profile(&self) -> &LocaleWeekProfile {
        &self.0.week
    }
}

pub fn embedded_native_locale_information_data_image(
) -> Result<NativeLocaleInformationDataImage, IntlDataImageError> {
    static IMAGE: OnceLock<Result<NativeLocaleInformationDataImage, IntlDataImageError>> =
        OnceLock::new();
    IMAGE
        .get_or_init(|| {
            NativeLocaleInformationDataImage::for_profile(
                IntlDataProfile::Minimal,
                &crate::embedded_locale_data_image()?,
                &crate::embedded_date_time_data_image()?,
            )
        })
        .clone()
}

#[cfg(test)]
mod tests;

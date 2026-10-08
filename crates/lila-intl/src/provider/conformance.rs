//! Complete source admission precedes the Conformance capability plan.

use super::*;

/// Private publication proof over the actual unprojected twelve-owner group.
/// Component constructors have already checked schemas, all typed rows and
/// mandatory marker/model/kernel dependencies. Four authorities have no
/// projected constructor; the remaining eight must retain their full pinned
/// physical payload, including associations which are not publicly advertised.
pub(super) struct CheckedConformanceData(());

impl CheckedConformanceData {
    pub(super) fn new(
        locale: &LocaleDataImage,
        lists: &ListDataImage,
        collators: &CollatorDataImage,
        numbers: &NumberProfilesDataImage,
        segmenters: &SegmenterDataImage,
        display_names: &DisplayNamesDataImage,
        relative_times: &RelativeTimeDataImage,
        durations: &DurationDataImage,
        named_zones: &NamedTimeZoneDataImage,
        date_time: &DateTimeDataImage,
        time_zone_names: &TimeZoneNamesDataImage,
        locale_information: &NativeLocaleInformationDataImage,
    ) -> Result<Self, IntlDataImageError> {
        if [
            locale.profile(),
            lists.profile(),
            collators.profile(),
            numbers.profile(),
            segmenters.profile(),
            display_names.profile(),
            relative_times.profile(),
            durations.profile(),
            named_zones.profile(),
            date_time.profile(),
            time_zone_names.profile(),
            locale_information.profile(),
        ]
        .iter()
        .any(|profile| **profile != crate::IntlDataProfile::Conformance)
        {
            return Err(IntlDataImageError::IncompleteConformance);
        }
        lists.require_complete_source()?;
        collators.require_complete_source()?;
        numbers.require_complete_source()?;
        segmenters.require_complete_source()?;
        display_names.require_complete_source()?;
        relative_times.require_complete_source()?;
        durations.require_complete_source()?;
        date_time.require_complete_source()?;
        Ok(Self(()))
    }

    pub(super) fn into_plan(self) -> IntlProfilePlan {
        IntlProfilePlan::conformance()
    }
}

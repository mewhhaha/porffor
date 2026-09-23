//! Checked CLDR profiles, exact calendar fields and one parts renderer.

use crate::datetime::*;

use super::named_time_zones::NamedTimeZones;

mod calendar;
mod identity;
mod locale;
mod names;
mod pattern;
mod plan;
mod profile;
mod ranges;
mod raw;
mod render;
#[cfg(test)]
mod tests;
mod validation;
mod zones;

use profile::Profile;

pub(super) const PROVIDER_DATA_SHA256: [u8; 32] = identity::PROVIDER_DATA_SHA256;
const PLAN_MAGIC: &[u8; 8] = b"LILADTF1";

pub(super) struct DateTimeProvider {
    profile: Profile,
}

impl DateTimeProvider {
    pub(super) fn from_pinned_data() -> Result<Self, DateTimeFormatError> {
        Ok(Self {
            profile: Profile::from_json(include_str!("datetime/generated/profile.json"))?,
        })
    }

    pub(super) fn resolve_locale(
        &self,
        request: DateTimeLocaleRequest,
    ) -> Result<DateTimeLocaleResult, DateTimeFormatError> {
        locale::resolve(&self.profile, request)
    }
    pub(super) fn supported_locales(
        &self,
        request: DateTimeSupportedLocalesRequest,
    ) -> Result<DateTimeSupportedLocalesResult, DateTimeFormatError> {
        Ok(locale::supported(&self.profile, request))
    }
    pub(super) fn select_plan(
        &self,
        request: DateTimePlanRequest,
    ) -> Result<DateTimePlanResult, DateTimeFormatError> {
        let selected = plan::select(&self.profile, &request)?;
        let recipe = request.encode().map_err(plan_wire_error)?;
        let mut bytes =
            Vec::with_capacity(PLAN_MAGIC.len() + PROVIDER_DATA_SHA256.len() + recipe.len());
        bytes.extend_from_slice(PLAN_MAGIC);
        bytes.extend_from_slice(&PROVIDER_DATA_SHA256);
        bytes.extend_from_slice(&recipe);
        Ok(DateTimePlanResult {
            plan: EncodedDateTimePlan::from_bytes(bytes),
            locale: request.locale,
            time_zone: request.time_zone,
            components: selected.legacy_components(),
            styles: match request.selection {
                DateTimeStyleSelection::Components(_) => None,
                DateTimeStyleSelection::Styles(styles) => Some(styles),
            },
            available_formats: selected.available_formats(),
        })
    }
    pub(super) fn format_parts(
        &self,
        request: DateTimeFormatRequest,
        named_time_zones: &NamedTimeZones,
    ) -> Result<DateTimeParts, DateTimeFormatError> {
        let recipe = self.decode_plan(&request.plan)?;
        let selected = plan::select(&self.profile, &recipe)?;
        render::format(&self.profile, &selected, request.input, named_time_zones)
    }
    pub(super) fn format_range_parts(
        &self,
        request: DateTimeRangeRequest,
        named_time_zones: &NamedTimeZones,
    ) -> Result<DateTimeRangeParts, DateTimeFormatError> {
        if request.start.kind() != request.end.kind() {
            return Err(DateTimeFormatError::InputKindMismatch);
        }
        let recipe = self.decode_plan(&request.plan)?;
        let selected = plan::select(&self.profile, &recipe)?;
        ranges::format(
            &self.profile,
            &selected,
            request.start,
            request.end,
            named_time_zones,
        )
    }
    fn decode_plan(
        &self,
        plan: &EncodedDateTimePlan,
    ) -> Result<DateTimePlanRequest, DateTimeFormatError> {
        let bytes = plan.as_bytes();
        if bytes.get(..8) != Some(PLAN_MAGIC.as_slice())
            || bytes.get(8..40) != Some(PROVIDER_DATA_SHA256.as_slice())
        {
            return Err(DateTimeFormatError::InvalidPlan(
                "schema or provider identity mismatch",
            ));
        }
        DateTimePlanRequest::decode(&bytes[40..]).map_err(plan_wire_error)
    }
}

fn plan_wire_error(error: crate::DateTimeWireError) -> DateTimeFormatError {
    match error {
        crate::DateTimeWireError::Domain(error) => error,
        crate::DateTimeWireError::Malformed(reason)
        | crate::DateTimeWireError::Resource(reason) => DateTimeFormatError::InvalidPlan(reason),
    }
}

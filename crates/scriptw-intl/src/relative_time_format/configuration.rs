use super::{
    profiles::Locale, RelativeNumeric, RelativeProfiles, RelativeStyle, RelativeTimeError,
};
use crate::number_format::numeric::RoundingSettings;
use crate::number_format::options::*;
use crate::number_format::{
    owned_text, NumberFormatConfiguration, NumberLocaleRequest, NumberProfiles, PartitionLimits,
    ResolvedNumberLocale,
};
use crate::plural_rules::{CheckedPluralConfiguration, PluralType, ResolvedPluralLocale};
use crate::CanonicalLocaleId;
use std::sync::Arc;

pub(super) fn canonical_copy(
    source: &str,
    limits: &PartitionLimits,
) -> Result<CanonicalLocaleId, RelativeTimeError> {
    CanonicalLocaleId::from_data(owned_text(source, limits)?)
        .map_err(|_| RelativeTimeError::InvalidLocale)
}

/// A shared numeric locale tied to one checked relative-time profile.
#[derive(Debug, Clone)]
pub struct ResolvedRelativeTimeLocale {
    number: ResolvedNumberLocale,
    profile: Arc<Locale>,
}
impl PartialEq for ResolvedRelativeTimeLocale {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number && Arc::ptr_eq(&self.profile, &other.profile)
    }
}
impl Eq for ResolvedRelativeTimeLocale {}
impl ResolvedRelativeTimeLocale {
    pub fn from_resolved(
        number: ResolvedNumberLocale,
        profiles: &RelativeProfiles,
    ) -> Result<Self, RelativeTimeError> {
        number.ensure_profiles(profiles.number_profiles())?;
        let profile = profiles
            .locale(number.formatting().as_str())
            .ok_or(RelativeTimeError::InvalidLocale)?
            .clone();
        Ok(Self { number, profile })
    }
    pub(super) fn ensure_owner(
        &self,
        profiles: &RelativeProfiles,
        numbers: &Arc<NumberProfiles>,
    ) -> Result<(), RelativeTimeError> {
        self.number.ensure_profiles(numbers)?;
        if !profiles
            .locale(self.formatting().as_str())
            .is_some_and(|profile| Arc::ptr_eq(profile, &self.profile))
        {
            return Err(RelativeTimeError::InvalidLocale);
        }
        Ok(())
    }
    pub fn resolved(&self) -> &CanonicalLocaleId {
        self.number.resolved()
    }
    pub fn formatting(&self) -> &CanonicalLocaleId {
        self.number.formatting()
    }
    pub fn numbering_system(&self) -> &str {
        self.number.numbering_system().name()
    }
}

impl RelativeProfiles {
    pub fn resolve_locale(
        &self,
        request: &NumberLocaleRequest,
        number_profiles: &Arc<NumberProfiles>,
        limits: &PartitionLimits,
    ) -> Result<ResolvedRelativeTimeLocale, RelativeTimeError> {
        self.ensure_number_profiles(number_profiles)?;
        if request.requested.len() as u128 > u128::from(limits.part_count()) {
            return Err(RelativeTimeError::Resource("locale count"));
        }
        // Select from this service's captured locales before using
        // the larger shared NumberFormat catalogue for nu negotiation.
        let selected = request.requested.iter().find_map(|locale| {
            self.matching_locale(locale.as_str(), request.matcher)
                .map(|base| (base, locale.as_str()))
        });
        let (base, original) = selected.unwrap_or(("en-US", "en-US"));
        let public_tag = original.split("-x-").next().unwrap_or(original);
        let extension = public_tag
            .find("-u-")
            .map(|start| &public_tag[start..])
            .unwrap_or("");
        let mut canonical = String::new();
        let length = base
            .len()
            .checked_add(extension.len())
            .ok_or(RelativeTimeError::Resource("locale extent"))?;
        if length as u128 > u128::from(limits.output_bytes()) {
            return Err(RelativeTimeError::Resource("locale extent"));
        }
        canonical
            .try_reserve_exact(length)
            .map_err(|_| RelativeTimeError::Resource("locale allocation"))?;
        canonical.push_str(base);
        canonical.push_str(extension);
        let selected_request = NumberLocaleRequest {
            requested: vec![canonical_copy(&canonical, limits)?].into_boxed_slice(),
            matcher: request.matcher,
            numbering_system: request.numbering_system.clone(),
        };
        let number = crate::number_format::resolve_number_locale_in(
            &selected_request,
            crate::number_format::NumberLocaleView::relative(number_profiles),
        )?;
        ResolvedRelativeTimeLocale::from_resolved(number, self)
    }
}

fn number_options() -> NumberFormatOptions {
    NumberFormatOptions {
        style: NumberStyle::Decimal,
        notation: Notation::Standard,
        minimum_integer_digits: IntegerDigitCount::new(1).expect("fixed RTF integer precision"),
        precision: Precision::Fraction(FractionPrecision::Range(
            FractionDigitRange::new(
                FractionDigitCount::new(0).expect("fixed RTF minimum fraction"),
                FractionDigitCount::new(3).expect("fixed RTF maximum fraction"),
            )
            .expect("ordered RTF fraction precision"),
        )),
        rounding_mode: RoundingMode::HalfExpand,
        trailing_zero_display: TrailingZeroDisplay::Auto,
        grouping: Grouping::Auto,
        sign_display: SignDisplay::Auto,
    }
}

/// Precision and cardinal selection share the exact NumberFormat rounding owner.
/// Public consumers cannot substitute options or an unrelated profile after validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeTimeConfiguration {
    locale: ResolvedRelativeTimeLocale,
    style: RelativeStyle,
    numeric: RelativeNumeric,
    number: NumberFormatConfiguration,
    plural: CheckedPluralConfiguration,
}
impl RelativeTimeConfiguration {
    pub fn new(
        locale: ResolvedRelativeTimeLocale,
        style: RelativeStyle,
        numeric: RelativeNumeric,
        number_profiles: &Arc<NumberProfiles>,
    ) -> Result<Self, RelativeTimeError> {
        locale.number.ensure_profiles(number_profiles)?;
        let options = number_options();
        let plural_locale = ResolvedPluralLocale::from_resolved_in(
            locale.formatting().clone(),
            locale.formatting().clone(),
            crate::number_format::NumberLocaleView::relative(number_profiles),
        )?;
        let plural = CheckedPluralConfiguration::new(
            plural_locale,
            PluralType::Cardinal,
            Notation::Standard,
            RoundingSettings::from(&options),
        );
        let number = NumberFormatConfiguration {
            locale: locale.number.clone(),
            options,
        };
        Ok(Self {
            locale,
            style,
            numeric,
            number,
            plural,
        })
    }
    pub fn locale(&self) -> &ResolvedRelativeTimeLocale {
        &self.locale
    }
    pub const fn style(&self) -> RelativeStyle {
        self.style
    }
    pub const fn numeric(&self) -> RelativeNumeric {
        self.numeric
    }
    pub(super) fn field(&self, unit: super::RelativeUnit) -> &super::profiles::Field {
        self.locale.profile.field(unit, self.style)
    }
    pub(super) fn number(&self) -> &NumberFormatConfiguration {
        &self.number
    }
    pub(super) fn plural(&self) -> &CheckedPluralConfiguration {
        &self.plural
    }
}

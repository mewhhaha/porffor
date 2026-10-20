use super::{
    profiles::Locale, DurationDisplay, DurationError, DurationFractionalDigits, DurationProfiles,
    DurationStyle, DurationUnit, DurationUnitStyle,
};
use crate::list_format::{CheckedListConfiguration, ListStyle, ListType};
use crate::number_format::{NumberLocaleRequest, NumberProfiles, ResolvedNumberLocale};
use crate::CanonicalLocaleId;
use std::sync::Arc;
#[derive(Debug, Clone)]
pub struct ResolvedDurationLocale {
    pub(super) number: ResolvedNumberLocale,
    pub(super) profile: Arc<Locale>,
}
impl ResolvedDurationLocale {
    pub fn resolved(&self) -> &CanonicalLocaleId {
        self.number.resolved()
    }
    pub fn two_digit_hours(&self) -> bool {
        self.profile.two_digit_hours
    }
    pub fn numbering_system(&self) -> &str {
        self.number.numbering_system().name()
    }
}
impl DurationProfiles {
    pub fn resolve_locale(
        &self,
        request: &NumberLocaleRequest,
        numbers: &Arc<NumberProfiles>,
    ) -> Result<ResolvedDurationLocale, DurationError> {
        self.ensure_numbers(numbers)?;
        let selected = request
            .requested
            .iter()
            .find_map(|locale| self.matching(locale.as_str()).map(|p| (p, locale.as_str())));
        let (profile, original) = selected
            .unwrap_or_else(|| (self.matching("en-US").expect("admitted default"), "en-US"));
        let public = original.split("-x-").next().unwrap_or(original);
        let extension = public.find("-u-").map(|i| &public[i..]).unwrap_or("");
        let text = format!("{}{}", profile.name.as_str(), extension);
        let canonical =
            CanonicalLocaleId::from_data(text).map_err(|_| DurationError::InvalidLocale)?;
        let selected = NumberLocaleRequest {
            requested: vec![canonical].into_boxed_slice(),
            matcher: request.matcher,
            numbering_system: request.numbering_system.clone(),
        };
        let number = crate::number_format::resolve_number_locale_in(
            &selected,
            crate::number_format::NumberLocaleView::duration(numbers),
        )?;
        if number.formatting() != &profile.name {
            return Err(DurationError::InvalidLocale);
        }
        Ok(ResolvedDurationLocale {
            number,
            profile: Arc::clone(profile),
        })
    }
    pub fn supported_locales(
        &self,
        request: super::DurationSupportedLocalesRequest,
    ) -> Box<[CanonicalLocaleId]> {
        match request.matcher {
            crate::number_format::options::LocaleMatcher::Lookup
            | crate::number_format::options::LocaleMatcher::BestFit => {}
        }
        let requested = &request.requested;
        requested
            .iter()
            .filter(|locale| self.matching(locale.as_str()).is_some())
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DurationUnitOption {
    pub style: Option<DurationUnitStyle>,
    pub display: Option<DurationDisplay>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationOptions {
    pub style: DurationStyle,
    pub units: [DurationUnitOption; 10],
    pub fractional_digits: Option<DurationFractionalDigits>,
}
impl Default for DurationOptions {
    fn default() -> Self {
        Self {
            style: DurationStyle::Short,
            units: [DurationUnitOption::default(); 10],
            fractional_digits: None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EffectiveStyle {
    Text(DurationUnitStyle),
    Numeric { two_digit: bool },
    Fractional,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EffectiveUnit {
    pub style: EffectiveStyle,
    pub display: DurationDisplay,
}
#[derive(Debug, Clone)]
pub struct CheckedDurationConfiguration {
    pub(super) locale: ResolvedDurationLocale,
    pub(super) base_style: DurationStyle,
    pub(super) units: [EffectiveUnit; 10],
    pub(super) fractional_digits: Option<DurationFractionalDigits>,
    pub(super) list: CheckedListConfiguration,
}
impl CheckedDurationConfiguration {
    pub fn new(
        locale: ResolvedDurationLocale,
        options: DurationOptions,
    ) -> Result<Self, DurationError> {
        let mut units = [EffectiveUnit {
            style: EffectiveStyle::Text(DurationUnitStyle::Short),
            display: DurationDisplay::Auto,
        }; 10];
        let mut previous = None;
        for &unit in DurationUnit::ALL {
            let index = unit.index();
            let request = options.units[index];
            if request.style.is_some_and(|s| {
                matches!(s, DurationUnitStyle::Numeric | DurationUnitStyle::TwoDigit)
            }) && (index < 4
                || (index > 6 && request.style == Some(DurationUnitStyle::TwoDigit)))
            {
                return Err(DurationError::InvalidOptions);
            }
            let (style, default_display) = if let Some(s) = request.style {
                (s, DurationDisplay::Always)
            } else if options.style == DurationStyle::Digital {
                (
                    if index >= 4 {
                        DurationUnitStyle::Numeric
                    } else {
                        DurationUnitStyle::Short
                    },
                    if (4..=6).contains(&index) {
                        DurationDisplay::Always
                    } else {
                        DurationDisplay::Auto
                    },
                )
            } else if matches!(
                previous,
                Some(EffectiveStyle::Numeric { .. } | EffectiveStyle::Fractional)
            ) {
                (
                    DurationUnitStyle::Numeric,
                    if matches!(unit, DurationUnit::Minute | DurationUnit::Second) {
                        DurationDisplay::Always
                    } else {
                        DurationDisplay::Auto
                    },
                )
            } else {
                (
                    match options.style {
                        DurationStyle::Long => DurationUnitStyle::Long,
                        DurationStyle::Short => DurationUnitStyle::Short,
                        DurationStyle::Narrow => DurationUnitStyle::Narrow,
                        DurationStyle::Digital => unreachable!(),
                    },
                    DurationDisplay::Auto,
                )
            };
            let fractional = index >= 7 && style == DurationUnitStyle::Numeric;
            let display = request.display.unwrap_or(if fractional {
                DurationDisplay::Auto
            } else {
                default_display
            });
            let effective = if fractional {
                EffectiveStyle::Fractional
            } else if matches!(
                style,
                DurationUnitStyle::Numeric | DurationUnitStyle::TwoDigit
            ) {
                EffectiveStyle::Numeric {
                    two_digit: style == DurationUnitStyle::TwoDigit,
                }
            } else {
                EffectiveStyle::Text(style)
            };
            let effective =
                validated_width(unit, effective, display, previous, locale.two_digit_hours())?;
            units[index] = EffectiveUnit {
                style: effective,
                display,
            };
            if (4..=8).contains(&index) {
                previous = Some(effective)
            }
        }
        let resolved_list = locale.profile.list.clone();
        let style = match options.style {
            DurationStyle::Long => ListStyle::Long,
            DurationStyle::Short | DurationStyle::Digital => ListStyle::Short,
            DurationStyle::Narrow => ListStyle::Narrow,
        };
        Ok(Self {
            locale,
            base_style: options.style,
            units,
            fractional_digits: options.fractional_digits,
            list: CheckedListConfiguration::new(resolved_list, ListType::Unit, style),
        })
    }
    pub fn locale(&self) -> &ResolvedDurationLocale {
        &self.locale
    }
    pub const fn style(&self) -> DurationStyle {
        self.base_style
    }
    pub const fn fractional_digits(&self) -> Option<DurationFractionalDigits> {
        self.fractional_digits
    }
    pub fn unit_options(&self, unit: DurationUnit) -> (DurationUnitStyle, DurationDisplay) {
        let row = self.units[unit.index()];
        (
            match row.style {
                EffectiveStyle::Text(style) => style,
                EffectiveStyle::Numeric { two_digit: false } | EffectiveStyle::Fractional => {
                    DurationUnitStyle::Numeric
                }
                EffectiveStyle::Numeric { two_digit: true } => DurationUnitStyle::TwoDigit,
            },
            row.display,
        )
    }
    pub(super) fn fractional(&self) -> [bool; 10] {
        self.units
            .map(|row| row.style == EffectiveStyle::Fractional)
    }
}

/// ValidateDurationUnitStyle precedes the normative hours/minutes/seconds overrides.
pub(super) fn validated_width(
    unit: DurationUnit,
    style: EffectiveStyle,
    display: DurationDisplay,
    previous: Option<EffectiveStyle>,
    two_digit_hours: bool,
) -> Result<EffectiveStyle, DurationError> {
    let fractional = style == EffectiveStyle::Fractional;
    if fractional && display == DurationDisplay::Always
        || matches!(previous, Some(EffectiveStyle::Fractional)) && !fractional
        || matches!(previous, Some(EffectiveStyle::Numeric { .. }))
            && matches!(style, EffectiveStyle::Text(_))
    {
        return Err(DurationError::InvalidOptions);
    }
    if unit == DurationUnit::Hour && two_digit_hours {
        return Ok(EffectiveStyle::Numeric { two_digit: true });
    }
    if matches!(unit, DurationUnit::Minute | DurationUnit::Second)
        && matches!(previous, Some(EffectiveStyle::Numeric { .. }))
    {
        return Ok(EffectiveStyle::Numeric { two_digit: true });
    }
    Ok(style)
}

use super::configuration::NumberFormatConfiguration;
use super::numeric::{
    format_decimal, DecimalFormatSettings, FiniteValue, IntlMathematicalValue, NotationScaling,
    NumberRange, NumberRangeEndpoint, NumberSign, RoundedDecimal, SelectedNotation,
};
use super::options::*;
use super::partition_resource::{
    owned_text, NumberFormatKernelError, NumberPartitionResourceError, PartitionLimits,
};
use super::parts::*;
use super::plural_rules::{CardinalCategory, PluralSelectionPurpose};
use super::profiles::*;

mod buffer;
mod range;
mod render;
use buffer::{bidi, digit_text, Owner, Piece, Pieces};
pub use range::partition_number_range;

#[derive(Clone, Copy)]
enum Input<'a> {
    Finite(FiniteValue<'a>),
    Infinity(NumberSign),
    NaN,
}
impl<'a> From<&'a IntlMathematicalValue> for Input<'a> {
    fn from(value: &'a IntlMathematicalValue) -> Self {
        match value {
            IntlMathematicalValue::Finite(value) => Self::Finite(FiniteValue::Nonzero(value)),
            IntlMathematicalValue::Zero(sign) => Self::Finite(FiniteValue::Zero(*sign)),
            IntlMathematicalValue::Infinity(sign) => Self::Infinity(*sign),
            IntlMathematicalValue::NaN => Self::NaN,
        }
    }
}
impl<'a> From<&'a NumberRangeEndpoint> for Input<'a> {
    fn from(value: &'a NumberRangeEndpoint) -> Self {
        match value {
            NumberRangeEndpoint::Finite(value) => Self::Finite(FiniteValue::Nonzero(value)),
            NumberRangeEndpoint::Zero(sign) => Self::Finite(FiniteValue::Zero(*sign)),
            NumberRangeEndpoint::Infinity(sign) => Self::Infinity(*sign),
        }
    }
}

struct FormatContext<'a> {
    options: &'a NumberFormatOptions,
    profiles: &'a NumberProfiles,
    locale: &'a LocaleProfile,
    numbering: &'a NumberingProfile,
    system: &'a NumberingSystem,
    symbols: &'a NumberSymbols,
    limits: &'a PartitionLimits,
}

impl<'a> FormatContext<'a> {
    fn new(
        configuration: &'a NumberFormatConfiguration,
        profiles: &'a NumberProfiles,
        limits: &'a PartitionLimits,
    ) -> Result<Self, NumberFormatKernelError> {
        let locale = profiles
            .profile(configuration.locale.formatting().as_str())
            .ok_or(NumberFormatKernelError::InvalidResolvedLocale)?;
        let (index, system) = profiles
            .system(configuration.locale.numbering_system().name())
            .ok_or(NumberFormatKernelError::InvalidResolvedLocale)?;
        let numbering = profiles.numbering(locale.numbering[index]);
        Ok(Self {
            options: &configuration.options,
            profiles,
            locale,
            numbering,
            system,
            symbols: profiles.symbols(numbering.symbols),
            limits,
        })
    }

    fn compact_set(&self) -> &CompactSet {
        let id = match (&self.options.style, self.options.notation) {
            (
                NumberStyle::Currency {
                    display:
                        CurrencyDisplay::Code | CurrencyDisplay::Symbol | CurrencyDisplay::NarrowSymbol,
                    ..
                },
                _,
            ) => self.numbering.compact_currency,
            (_, Notation::Compact(CompactDisplay::Long)) => self.numbering.compact_long,
            (
                _,
                Notation::Standard
                | Notation::Scientific
                | Notation::Engineering
                | Notation::Compact(CompactDisplay::Short),
            ) => self.numbering.compact_short,
        };
        self.profiles.compact(id)
    }

    fn currency_override(&self) -> Option<&CurrencyOverride> {
        match &self.options.style {
            NumberStyle::Currency { code, .. } => {
                self.numbering.currency_override(code.clone().ascii())
            }
            NumberStyle::Decimal | NumberStyle::Percent | NumberStyle::Unit { .. } => None,
        }
    }
}

struct Formatted {
    pieces: Pieces,
    category: CardinalCategory,
    approximate_at: usize,
}

pub fn partition_number(
    configuration: &NumberFormatConfiguration,
    value: &IntlMathematicalValue,
    profiles: &NumberProfiles,
    limits: &PartitionLimits,
) -> Result<ScalarNumberPartition, NumberFormatKernelError> {
    let context = FormatContext::new(configuration, profiles, limits)?;
    let formatted = context.format(value.into(), None)?;
    let mut parts = Vec::new();
    parts
        .try_reserve_exact(formatted.pieces.rows.len())
        .map_err(|_| NumberPartitionResourceError::Allocation)?;
    for piece in formatted.pieces.rows {
        parts.push(piece.part);
    }
    Ok(ScalarNumberPartition::from_parts(parts.into_boxed_slice()))
}

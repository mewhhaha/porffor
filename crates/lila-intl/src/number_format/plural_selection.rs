//! Rounded source-number operands for Intl.PluralRules.
//!
//! PluralRules applies FormatNumericToString directly to the source value. It
//! shares NumberFormat's exact digit rounding, but it does not rescale the
//! input for scientific, engineering, or compact notation. Compact notation
//! contributes only its selected exponent to operand `c`.

use core::cmp::Ordering;
use core::num::NonZeroU64;

use icu_plurals::{PluralOperands, RawPluralOperands};

use super::{
    configuration::ResolvedNumberLocale,
    numeric::{
        format_decimal, DecimalFormatSettings, DecimalScale, ExactPluralOperand,
        IntlMathematicalValue, NotationScaling, NumberFormatResourceError, PluralOperand,
    },
    options::{CompactDisplay, Notation, NumberFormatOptions},
    partition_resource::NumberFormatKernelError,
    partition_resource::PartitionLimits,
    profiles::NumberProfiles,
};

/// The inputs needed by PluralRuleSelect. `formatted_string` is the unsigned,
/// unlocalized FormatNumericToString result used by ResolvePluralRange's
/// equal-endpoint shortcut.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PluralSelectionInput {
    pub(crate) operands: PluralOperands,
    pub(crate) formatted_string: Box<[u8]>,
}

/// Returns rounded plural operands and the corresponding FormatNumericToString
/// result. `None` represents NaN or infinity, whose category is always other.
pub(crate) fn plural_operands_for_selection(
    value: &IntlMathematicalValue,
    locale: &ResolvedNumberLocale,
    options: &NumberFormatOptions,
    profiles: &NumberProfiles,
) -> Result<Option<PluralSelectionInput>, NumberFormatKernelError> {
    let Some(finite) = value.finite_value() else {
        return Ok(None);
    };

    let locale_profile = profiles
        .profile(locale.formatting().as_str())
        .ok_or(NumberFormatKernelError::InvalidResolvedLocale)?;
    let (numbering_index, _) = profiles
        .system(locale.numbering_system().name())
        .ok_or(NumberFormatKernelError::InvalidResolvedLocale)?;
    let numbering_id = *locale_profile
        .numbering
        .get(numbering_index)
        .ok_or(NumberFormatKernelError::InvalidResolvedLocale)?;
    let numbering = profiles.numbering(numbering_id);
    let compact = profiles.compact(match options.notation {
        Notation::Compact(CompactDisplay::Long) => numbering.compact_long,
        Notation::Standard
        | Notation::Scientific
        | Notation::Engineering
        | Notation::Compact(CompactDisplay::Short) => numbering.compact_short,
    });

    let mut source_settings = DecimalFormatSettings::from_options(options, &compact.exponents);
    source_settings.scale = DecimalScale::Unit;
    source_settings.notation = NotationScaling::Standard;
    let rounded = format_decimal(
        finite,
        &source_settings,
        &PartitionLimits::HOST_ABI.numeric(),
    )?;

    // ResolvePlural passes only this rounded string into PluralRuleSelect.
    // Select compact operand c from that same numeric value so two inputs that
    // round to the same FormattedString cannot choose different plural rules.
    let compact_exponent = if matches!(options.notation, Notation::Compact(_)) {
        let compact_settings = DecimalFormatSettings::from_options(options, &compact.exponents);
        format_decimal(
            rounded.rounded_value(),
            &compact_settings,
            &PartitionLimits::HOST_ABI.numeric(),
        )?
        .notation()
        .compact_exponent()
    } else {
        0
    };

    let formatted_string = format_numeric_identity(&rounded)?;
    let operands = rounded_source_operands(&rounded, compact_exponent);
    Ok(Some(PluralSelectionInput {
        operands,
        formatted_string,
    }))
}

const LARGE_OPERAND_BASE: u64 = 1_000_000_000_000_000_000;

/// ICU4X exposes u64 raw operands. CLDR 47's cardinal rules compare operands
/// only with constants <= 1,000,000 and use modulo divisors 10, 100, 1,000,
/// 100,000, and 1,000,000. Its ordinal rules compare only with constants <=
/// 1,000 and use modulo divisors 10, 100, and 1,000. All those divisors divide
/// 10^18. Encode a value with discarded nonzero high digits as 10^18 plus its
/// low 18 digits: direct comparisons stay greater than every rule bound while
/// all supported modulo results remain exact.
fn pack_cldr_operand(operand: ExactPluralOperand<'_>) -> u64 {
    let divisor = NonZeroU64::new(LARGE_OPERAND_BASE).expect("the CLDR operand base is nonzero");
    let suffix = operand
        .modulo(divisor)
        .small_integer_value()
        .expect("a modulo-reduced integral operand fits in u64");
    if operand.compare_integer(LARGE_OPERAND_BASE) != Ordering::Less {
        LARGE_OPERAND_BASE + suffix
    } else {
        suffix
    }
}

fn rounded_source_operands(
    rounded: &super::numeric::RoundedDecimal,
    compact_exponent: u32,
) -> PluralOperands {
    let exact = rounded.plural_operands();
    let v = exact
        .operand(PluralOperand::V)
        .small_integer_value()
        .expect("visible fraction count is an integral operand");
    let w = exact
        .operand(PluralOperand::W)
        .small_integer_value()
        .expect("trimmed fraction count is an integral operand");
    RawPluralOperands {
        i: pack_cldr_operand(exact.operand(PluralOperand::I)),
        v: usize::try_from(v).expect("number operand fits usize"),
        w: usize::try_from(w).expect("number operand fits usize"),
        f: pack_cldr_operand(exact.operand(PluralOperand::F)),
        t: pack_cldr_operand(exact.operand(PluralOperand::T)),
        c: usize::try_from(compact_exponent).unwrap_or(usize::MAX),
    }
    .into()
}

fn format_numeric_identity(
    rounded: &super::numeric::RoundedDecimal,
) -> Result<Box<[u8]>, NumberFormatResourceError> {
    let integer = rounded.integer_digits();
    let fraction = rounded.fraction_digits();
    let capacity = integer
        .len()
        .checked_add(usize::from(!fraction.is_empty()))
        .and_then(|length| length.checked_add(fraction.len()))
        .ok_or(NumberFormatResourceError::Allocation)?;
    let mut formatted = Vec::new();
    formatted
        .try_reserve_exact(capacity)
        .map_err(|_| NumberFormatResourceError::Allocation)?;
    formatted.extend(integer.iter().map(|digit| b'0' + digit));
    if !fraction.is_empty() {
        formatted.push(b'.');
        formatted.extend(fraction.iter().map(|digit| b'0' + digit));
    }
    Ok(formatted.into_boxed_slice())
}

use super::{
    configuration::EffectiveStyle, record::exact_decimal, CheckedDurationConfiguration,
    DurationDisplay, DurationError, DurationPart, DurationPartition, DurationRecord, DurationUnit,
    DurationUnitStyle,
};
use crate::list_format::{FormatListPartsRequest, ListPart, ListProfiles};
use crate::number_format::numeric::IntlMathematicalValue;
use crate::number_format::options::*;
use crate::number_format::{
    owned_text, partition_number, NumberFormatConfiguration, NumberPart, NumberPartKind,
    NumberProfiles, PartitionLimits,
};
use std::sync::Arc;
fn single(unit: DurationUnit) -> SingleUnit {
    match unit {
        DurationUnit::Year => SingleUnit::Year,
        DurationUnit::Month => SingleUnit::Month,
        DurationUnit::Week => SingleUnit::Week,
        DurationUnit::Day => SingleUnit::Day,
        DurationUnit::Hour => SingleUnit::Hour,
        DurationUnit::Minute => SingleUnit::Minute,
        DurationUnit::Second => SingleUnit::Second,
        DurationUnit::Millisecond => SingleUnit::Millisecond,
        DurationUnit::Microsecond => SingleUnit::Microsecond,
        DurationUnit::Nanosecond => SingleUnit::Nanosecond,
    }
}
fn number(
    configuration: &CheckedDurationConfiguration,
    value: &IntlMathematicalValue,
    unit: DurationUnit,
    textual: Option<DurationUnitStyle>,
    two_digit: bool,
    fractional: bool,
    sign: bool,
    numbers: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<Vec<DurationPart>, DurationError> {
    let (minimum, maximum) = if fractional {
        configuration
            .fractional_digits
            .map(|d| (d.value(), d.value()))
            .unwrap_or((0, 9))
    } else {
        (0, 3)
    };
    let precision = Precision::Fraction(FractionPrecision::Range(
        FractionDigitRange::new(
            FractionDigitCount::new(minimum).map_err(|_| DurationError::InvalidOptions)?,
            FractionDigitCount::new(maximum).map_err(|_| DurationError::InvalidOptions)?,
        )
        .map_err(|_| DurationError::InvalidOptions)?,
    ));
    let style = if let Some(style) = textual {
        NumberStyle::Unit {
            identifier: UnitIdentifier::Single(single(unit)),
            display: match style {
                DurationUnitStyle::Long => UnitDisplay::Long,
                DurationUnitStyle::Short => UnitDisplay::Short,
                DurationUnitStyle::Narrow => UnitDisplay::Narrow,
                DurationUnitStyle::Numeric | DurationUnitStyle::TwoDigit => {
                    return Err(DurationError::InvalidOptions)
                }
            },
        }
    } else {
        NumberStyle::Decimal
    };
    let options = NumberFormatOptions {
        style,
        notation: Notation::Standard,
        minimum_integer_digits: IntegerDigitCount::new(if two_digit { 2 } else { 1 })
            .map_err(|_| DurationError::InvalidOptions)?,
        precision,
        rounding_mode: if fractional {
            RoundingMode::Trunc
        } else {
            RoundingMode::HalfExpand
        },
        trailing_zero_display: TrailingZeroDisplay::Auto,
        grouping: if textual.is_some() {
            Grouping::Auto
        } else {
            Grouping::Never
        },
        sign_display: if sign {
            SignDisplay::Auto
        } else {
            SignDisplay::Never
        },
    };
    let partition = partition_number(
        &NumberFormatConfiguration {
            locale: configuration.locale.number.clone(),
            options,
        },
        value,
        numbers,
        limits,
    )?;
    partition
        .parts()
        .iter()
        .map(|part| {
            Ok(DurationPart {
                part: NumberPart::new(part.kind(), owned_text(part.text(), limits)?),
                unit: Some(unit),
            })
        })
        .collect()
}
fn separator(
    parts: &mut Vec<DurationPart>,
    text: &str,
    limits: &PartitionLimits,
) -> Result<(), DurationError> {
    parts.push(DurationPart {
        part: NumberPart::new(NumberPartKind::Literal, owned_text(text, limits)?),
        unit: None,
    });
    Ok(())
}
fn numeric(
    configuration: &CheckedDurationConfiguration,
    record: &DurationRecord,
    first: DurationUnit,
    mut sign: bool,
    numbers: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<Vec<DurationPart>, DurationError> {
    let fractional = configuration.fractional();
    let hours = first == DurationUnit::Hour
        && (record.magnitude(DurationUnit::Hour) != 0
            || configuration.units[4].display == DurationDisplay::Always);
    let seconds_value = record.fraction_value(DurationUnit::Second, &fractional, sign);
    let seconds = !matches!(&seconds_value, IntlMathematicalValue::Zero(_))
        || configuration.units[6].display == DurationDisplay::Always;
    let minutes = first != DurationUnit::Second
        && (hours && seconds
            || record.magnitude(DurationUnit::Minute) != 0
            || configuration.units[5].display == DurationDisplay::Always);
    let mut result = Vec::new();
    for (unit, printed) in [
        (DurationUnit::Hour, hours),
        (DurationUnit::Minute, minutes),
        (DurationUnit::Second, seconds),
    ] {
        if !printed {
            continue;
        }
        if unit == DurationUnit::Minute && hours {
            separator(
                &mut result,
                &configuration.locale.profile.hour_minute,
                limits,
            )?;
        }
        if unit == DurationUnit::Second && minutes {
            separator(
                &mut result,
                &configuration.locale.profile.minute_second,
                limits,
            )?;
        }
        let value = if unit == DurationUnit::Second {
            record.fraction_value(unit, &fractional, sign)
        } else {
            exact_decimal(record.magnitude(unit), 0, sign && record.negative())
        };
        let two_digit = match configuration.units[unit.index()].style {
            EffectiveStyle::Numeric { two_digit } => two_digit,
            EffectiveStyle::Text(_) | EffectiveStyle::Fractional => {
                return Err(DurationError::InvalidOptions)
            }
        };
        result.extend(number(
            configuration,
            &value,
            unit,
            None,
            two_digit,
            unit == DurationUnit::Second,
            sign,
            numbers,
            limits,
        )?);
        sign = false;
    }
    Ok(result)
}
fn checked(
    parts: Vec<DurationPart>,
    limits: &PartitionLimits,
) -> Result<DurationPartition, DurationError> {
    if parts.len() as u128 > u128::from(limits.part_count()) {
        return Err(DurationError::Resource("part extent"));
    }
    let bytes = parts
        .iter()
        .try_fold(0usize, |sum, part| sum.checked_add(part.text().len()))
        .ok_or(DurationError::Resource("text extent"))?;
    if bytes as u128 > u128::from(limits.output_bytes()) {
        return Err(DurationError::Resource("text extent"));
    }
    Ok(DurationPartition {
        parts: parts.into_boxed_slice(),
        bytes,
    })
}
pub(super) fn format(
    configuration: &CheckedDurationConfiguration,
    record: &DurationRecord,
    numbers: &Arc<NumberProfiles>,
    lists: &Arc<ListProfiles>,
    limits: &PartitionLimits,
) -> Result<DurationPartition, DurationError> {
    configuration.locale.number.ensure_profiles(numbers)?;
    let mut groups = Vec::<Vec<DurationPart>>::new();
    let mut sign = true;
    let fractional = configuration.fractional();
    for &unit in DurationUnit::ALL {
        let index = unit.index();
        let row = configuration.units[index];
        match row.style {
            EffectiveStyle::Numeric { .. } => {
                let group = numeric(configuration, record, unit, sign, numbers, limits)?;
                if !group.is_empty() {
                    groups.push(group)
                }
                break;
            }
            EffectiveStyle::Fractional => return Err(DurationError::InvalidOptions),
            EffectiveStyle::Text(style) => {
                let absorbs = fractional.get(index + 1) == Some(&true);
                let value = record.fraction_value(unit, &fractional, sign);
                if row.display == DurationDisplay::Always
                    || !matches!(&value, IntlMathematicalValue::Zero(_))
                {
                    groups.push(number(
                        configuration,
                        &value,
                        unit,
                        Some(style),
                        false,
                        absorbs,
                        sign,
                        numbers,
                        limits,
                    )?);
                    sign = false;
                }
                if absorbs {
                    break;
                }
            }
        }
    }
    // Existing checked ListFormat assigns each element exactly once in source
    // order. Replace element references with complete number/unit partitions.
    let elements = groups
        .iter()
        .map(|group| {
            group
                .iter()
                .flat_map(|p| p.text().encode_utf16())
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let request = FormatListPartsRequest::new(configuration.list.clone(), elements)?;
    let list = lists.format_parts(request)?;
    let mut result = Vec::new();
    for part in list.parts() {
        match part {
            ListPart::Element(index) => result.extend(groups[*index as usize].iter().cloned()),
            ListPart::Literal(text) => {
                let text = String::from_utf16(text).map_err(|_| DurationError::InvalidProfile)?;
                separator(&mut result, &text, limits)?;
            }
        }
    }
    checked(result, limits)
}

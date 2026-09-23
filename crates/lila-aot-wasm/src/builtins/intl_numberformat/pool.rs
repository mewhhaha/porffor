use super::*;

pub(crate) fn intl_number_format_pool_strings() -> Vec<String> {
    let mut strings: Vec<_> = [
        "Intl.NumberFormat",
        "NumberFormat",
        "supportedLocalesOf",
        "resolvedOptions",
        "format",
        "formatToParts",
        "formatRange",
        "formatRangeToParts",
        "locale",
        "type",
        "value",
        "source",
        "approximatelySign",
        "",
        "-per-",
        "true",
        "false",
        "auto",
        "always",
        "min2",
        NF_RECEIVER_ERROR,
        NF_RANGE_UNDEFINED,
        NF_RANGE_NAN,
        NF_CURRENCY_REQUIRED,
        NF_UNIT_REQUIRED,
        NF_INCREMENT_PRECISION,
        NF_INCREMENT_RANGE,
        NF_DIGIT_RANGE,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for property in [
        "localeMatcher",
        "numberingSystem",
        "style",
        "currency",
        "currencyDisplay",
        "currencySign",
        "unit",
        "unitDisplay",
        "notation",
        "minimumIntegerDigits",
        "minimumFractionDigits",
        "maximumFractionDigits",
        "minimumSignificantDigits",
        "maximumSignificantDigits",
        "roundingIncrement",
        "roundingMode",
        "roundingPriority",
        "trailingZeroDisplay",
        "compactDisplay",
        "useGrouping",
        "signDisplay",
    ] {
        strings.push(property.to_owned());
        strings.push(format!("Invalid {property} option"));
    }
    for options in [
        LocaleMatcher::OPTIONS,
        StyleOption::OPTIONS,
        CurrencyDisplay::OPTIONS,
        CurrencySign::OPTIONS,
        UnitDisplay::OPTIONS,
        NotationOption::OPTIONS,
        CompactDisplay::OPTIONS,
        RoundingMode::OPTIONS,
        RoundingPriority::OPTIONS,
        TrailingZeroDisplay::OPTIONS,
        SignDisplay::OPTIONS,
    ] {
        strings.extend(options.iter().map(|(name, _)| (*name).to_owned()));
    }
    strings.extend(SingleUnit::ALL.iter().map(|unit| unit.name().to_owned()));
    strings.extend(
        NumberPartKind::ALL
            .iter()
            .map(|part| part.name().to_owned()),
    );
    strings.extend(
        RangePartSource::ALL
            .iter()
            .map(|source| source.name().to_owned()),
    );
    strings
}

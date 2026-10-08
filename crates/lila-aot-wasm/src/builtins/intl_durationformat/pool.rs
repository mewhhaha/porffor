use super::*;

pub(crate) fn intl_durationformat_pool_strings() -> Vec<String> {
    let mut strings = [
        "Intl.DurationFormat",
        "DurationFormat",
        "supportedLocalesOf",
        "resolvedOptions",
        "format",
        "formatToParts",
        "locale",
        "numberingSystem",
        "localeMatcher",
        "style",
        "fractionalDigits",
        "type",
        "value",
        "unit",
        "",
        DU_NATIVE_ERROR,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    strings.extend(
        LocaleMatcher::ALL
            .iter()
            .map(|matcher| matcher.name().to_owned()),
    );
    strings.extend(DurationStyle::ALL.iter().map(|s| s.name().to_owned()));
    strings.extend(DurationUnitStyle::ALL.iter().map(|s| s.name().to_owned()));
    strings.extend(DurationDisplay::ALL.iter().map(|s| s.name().to_owned()));
    for &unit in DurationUnit::ALL {
        let name = format!("{}s", unit.name());
        strings.extend([
            unit.name().to_owned(),
            name.clone(),
            format!("{name}Display"),
            format!("Invalid {name} option"),
            format!("Invalid {name}Display option"),
        ]);
    }
    strings.extend(
        [
            NumberPartKind::Literal,
            NumberPartKind::Integer,
            NumberPartKind::Group,
            NumberPartKind::Decimal,
            NumberPartKind::Fraction,
            NumberPartKind::MinusSign,
            NumberPartKind::Unit,
        ]
        .into_iter()
        .map(|kind| kind.name().to_owned()),
    );
    strings
}

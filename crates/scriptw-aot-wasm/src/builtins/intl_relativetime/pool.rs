use super::*;

pub(crate) fn intl_relative_time_pool_strings() -> Vec<String> {
    let mut strings: Vec<String> = [
        "Intl.RelativeTimeFormat",
        "RelativeTimeFormat",
        "supportedLocalesOf",
        "resolvedOptions",
        "format",
        "formatToParts",
        "locale",
        "numberingSystem",
        "localeMatcher",
        "style",
        "numeric",
        "type",
        "value",
        "unit",
        "",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    strings.extend(LocaleMatcher::ALL.iter().map(|v| v.name().to_owned()));
    strings.extend(RelativeStyle::ALL.iter().map(|v| v.name().to_owned()));
    strings.extend(RelativeNumeric::ALL.iter().map(|v| v.name().to_owned()));
    for unit in RelativeUnit::ALL {
        strings.push(unit.name().to_owned());
        strings.push(format!("{}s", unit.name()));
    }
    strings.extend(
        [
            NumberPartKind::Literal,
            NumberPartKind::Integer,
            NumberPartKind::Group,
            NumberPartKind::Decimal,
            NumberPartKind::Fraction,
        ]
        .into_iter()
        .map(|kind| kind.name().to_owned()),
    );
    strings
}

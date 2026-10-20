use super::*;
pub(crate) fn intl_plural_rules_pool_strings() -> Vec<String> {
    let mut strings = [
        "Intl.PluralRules",
        "PluralRules",
        "supportedLocalesOf",
        "resolvedOptions",
        "select",
        "selectRange",
        "locale",
        "type",
        "notation",
        "compactDisplay",
        "pluralCategories",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    strings.extend(PluralType::ALL.iter().map(|v| v.name().to_owned()));
    strings.extend(PluralCategory::ALL.iter().map(|v| v.name().to_owned()));
    strings
}

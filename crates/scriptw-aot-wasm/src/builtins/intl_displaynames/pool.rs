use super::*;
pub(crate) fn intl_display_names_pool_strings() -> Vec<String> {
    let mut values = [
        "Intl.DisplayNames",
        "DisplayNames",
        "supportedLocalesOf",
        "resolvedOptions",
        "of",
        "locale",
        "localeMatcher",
        "style",
        "type",
        "fallback",
        "languageDisplay",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();
    values.extend(LocaleMatcher::ALL.iter().map(|v| v.name().to_owned()));
    values.extend(DisplayNamesType::ALL.iter().map(|v| v.name().to_owned()));
    values.extend(DisplayNamesStyle::ALL.iter().map(|v| v.name().to_owned()));
    values.extend(
        DisplayNamesFallback::ALL
            .iter()
            .map(|v| v.name().to_owned()),
    );
    values.extend(
        DisplayNamesLanguageDisplay::ALL
            .iter()
            .map(|v| v.name().to_owned()),
    );
    values
}

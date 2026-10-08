use super::*;
pub(crate) fn intl_list_format_pool_strings() -> Vec<String> {
    let mut strings = [
        "Intl.ListFormat",
        "ListFormat",
        "supportedLocalesOf",
        "resolvedOptions",
        "format",
        "formatToParts",
        "locale",
        "type",
        "style",
        "value",
        "element",
        "literal",
        "",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    strings.extend(ListType::ALL.iter().map(|v| v.name().to_owned()));
    strings.extend(ListStyle::ALL.iter().map(|v| v.name().to_owned()));
    strings
}

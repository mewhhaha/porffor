use super::*;
pub(crate) fn intl_collator_pool_strings() -> Vec<String> {
    let mut strings = [
        "Intl.Collator",
        "Collator",
        "compare",
        "resolvedOptions",
        "supportedLocalesOf",
        "locale",
        "usage",
        "collation",
        "numeric",
        "caseFirst",
        "sensitivity",
        "ignorePunctuation",
        "default",
        "",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    strings.extend(CollatorUsage::ALL.iter().map(|v| v.name().to_owned()));
    strings.extend(CollatorSensitivity::ALL.iter().map(|v| v.name().to_owned()));
    strings.extend(CollatorCaseFirst::ALL.iter().map(|v| v.name().to_owned()));
    strings
}

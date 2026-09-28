use super::*;

pub(crate) fn intl_collator_pool_strings() -> Vec<String> {
    [
        "Intl.Collator",
        "Collator",
        "supportedLocalesOf",
        "resolvedOptions",
        "compare",
        "locale",
        "usage",
        "collation",
        "numeric",
        "caseFirst",
        "sensitivity",
        "ignorePunctuation",
        "sort",
        "search",
        "lookup",
        "best fit",
        "upper",
        "lower",
        "false",
        "base",
        "accent",
        "case",
        "variant",
        "default",
        "",
        "Invalid usage option",
        "Invalid localeMatcher option",
        "Invalid caseFirst option",
        "Invalid sensitivity option",
        "Invalid collation option",
        "Invalid Intl.Locale collation option",
        "Invalid language tag",
        COLLATOR_RECEIVER_ERROR,
        COLLATOR_INVALID_LOCALE,
        COLLATOR_INVALID_OPTION,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

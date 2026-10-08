use super::*;
pub(crate) fn intl_segmenter_pool_strings() -> Vec<String> {
    let mut values = [
        "Intl.Segmenter",
        "Segmenter",
        "Segmenter String Iterator",
        "supportedLocalesOf",
        "resolvedOptions",
        "segment",
        "containing",
        "next",
        "[Symbol.iterator]",
        "locale",
        "localeMatcher",
        "granularity",
        "index",
        "input",
        "isWordLike",
        "value",
        "done",
        "Invalid Intl.Segmenter request",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();
    values.extend(LocaleMatcher::ALL.iter().map(|v| v.name().to_owned()));
    values.extend(
        SegmenterGranularity::ALL
            .iter()
            .map(|v| v.name().to_owned()),
    );
    values
}

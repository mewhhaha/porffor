const PARENT_SOURCE: &str = include_str!("../src/builtins/regexp.rs");
const RANGE_SEARCH_SOURCE: &str = include_str!("../src/builtins/regexp/range_search.rs");
const RECURSIVE_SOURCE: &str = concat!(
    include_str!("../src/builtins/regexp.rs"),
    include_str!("../src/builtins/regexp/range_search.rs")
);

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn range_bound_offset_is_an_exhaustive_closed_projection() {
    let variants = bounded(
        RANGE_SEARCH_SOURCE,
        "enum RegExpRangeBound {",
        "impl RegExpRangeBound {",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty() && *line != "}")
    .collect::<Vec<_>>();
    assert_eq!(variants, ["Start,", "End,"]);

    let projection = normalized(bounded(
        RANGE_SEARCH_SOURCE,
        "impl RegExpRangeBound {",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    assert_eq!(projection.matches("Self::Start=>0").count(), 1);
    assert_eq!(projection.matches("Self::End=>4").count(), 1);
    assert_eq!(projection.matches("=>").count(), 2);
    assert!(!projection.contains("_=>"));
    assert!(!projection.contains("unreachable!"));
    assert_eq!(PARENT_SOURCE.matches("mod range_search;").count(), 1);
    assert!(!PARENT_SOURCE.contains("RegExpRangeBound"));
    assert!(!PARENT_SOURCE.contains("emit_regexp_range_bound_load("));
    assert!(!PARENT_SOURCE.contains("range_search::"));
    assert_eq!(RANGE_SEARCH_SOURCE.matches("RegExpRangeBound").count(), 5);
    assert_eq!(RANGE_SEARCH_SOURCE.matches("RegExpRangeBound::").count(), 2);
    assert_eq!(RECURSIVE_SOURCE.matches("RegExpRangeBound").count(), 5);
    assert_eq!(
        PARENT_SOURCE
            .matches("self.emit_regexp_unicode_property_mismatch(")
            .count(),
        2
    );
}

#[test]
fn range_bound_reader_accepts_only_the_closed_domain() {
    let signature = bounded(
        RANGE_SEARCH_SOURCE,
        "fn emit_regexp_range_bound_load(",
        ") {",
    );
    assert!(signature.contains("bound: RegExpRangeBound,"));
    assert!(!signature.contains("field: u64"));

    let reader = bounded(
        RANGE_SEARCH_SOURCE,
        "fn emit_regexp_range_bound_load(",
        "\n    }\n}",
    );
    assert_eq!(reader.matches("bound.offset()").count(), 1);
    assert!(!reader.contains("memarg32(field)"));
}

#[test]
fn range_search_reads_end_before_start_with_named_bounds() {
    let search = normalized(bounded(
        RANGE_SEARCH_SOURCE,
        "pub(super) fn emit_regexp_unicode_property_mismatch(",
        "/// Pushes the selected inclusive bound of range-pool entry",
    ));
    assert_eq!(search.matches("emit_regexp_range_bound_load(").count(), 2);
    assert_eq!(search.matches("RegExpRangeBound::End").count(), 1);
    assert_eq!(search.matches("RegExpRangeBound::Start").count(), 1);
    assert!(search.contains(concat!(
        "range_middle_local,RegExpRangeBound::End,function,);",
        "function.instruction(&Instruction::I64GtU);"
    )));
    assert!(search.contains(concat!(
        "range_low_local,RegExpRangeBound::Start,function,);",
        "function.instruction(&Instruction::I64GeU);"
    )));
    assert!(!search.contains("range_middle_local,4,function"));
    assert!(!search.contains("range_low_local,0,function"));
    assert_eq!(
        RANGE_SEARCH_SOURCE
            .matches("emit_regexp_range_bound_load(")
            .count(),
        3
    );
    assert_eq!(
        RECURSIVE_SOURCE
            .matches("emit_regexp_unicode_property_mismatch(")
            .count(),
        3
    );
}

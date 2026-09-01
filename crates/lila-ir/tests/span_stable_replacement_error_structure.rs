use std::fs;
use std::path::Path;

const OWNER_SOURCE: &str = include_str!("../src/modules/source.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn code_without_whitespace(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn replacement_failure_is_a_private_three_row_domain_without_incidental_capabilities() {
    assert_eq!(
        code_without_whitespace(bounded(
            OWNER_SOURCE,
            "struct SpanStableReplacement(String);",
            "impl SourceEdit {",
        )),
        "enumSpanStableReplacementError{InvalidSpan,GeneratedLineTerminator,DoesNotFit,}"
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("enum SpanStableReplacementError")
            .count(),
        1
    );
    assert!(!OWNER_SOURCE.contains("pub enum SpanStableReplacementError"));
    assert!(!OWNER_SOURCE.contains("impl SpanStableReplacementError"));
    assert!(!OWNER_SOURCE.contains("for SpanStableReplacementError"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "SpanStableReplacementError"),
        13
    );
    assert_eq!(
        OWNER_SOURCE.matches("SpanStableReplacementError").count(),
        13
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("SpanStableReplacementError::InvalidSpan")
            .count(),
        3
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("SpanStableReplacementError::GeneratedLineTerminator")
            .count(),
        3
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("SpanStableReplacementError::DoesNotFit")
            .count(),
        4
    );
}

#[test]
fn replacement_failure_producers_preserve_all_six_conditions_and_their_order() {
    let source_edit = code_without_whitespace(bounded(
        OWNER_SOURCE,
        "fn replace_around_padding(",
        "impl SpanStableReplacement {",
    ));
    assert_eq!(
        source_edit
            .matches("SpanStableReplacementError::InvalidSpan")
            .count(),
        2
    );
    let erased = "leterased=source.get(start..end)\
        .ok_or(SpanStableReplacementError::InvalidSpan)?;";
    let suffix = "letsuffix=source.get(end..)\
        .ok_or(SpanStableReplacementError::InvalidSpan)?;";
    let admitted = "letreplacement=SpanStableReplacement::around_padding(\
        erased,suffix,before_padding,after_padding)?;";
    for producer in [erased, suffix, admitted] {
        assert!(
            source_edit.contains(producer),
            "missing source-edit step `{producer}`"
        );
    }
    assert!(source_edit.find(erased).unwrap() < source_edit.find(suffix).unwrap());
    assert!(source_edit.find(suffix).unwrap() < source_edit.find(admitted).unwrap());

    let around_padding = code_without_whitespace(bounded(
        OWNER_SOURCE,
        "fn around_padding(",
        "fn as_str(&self) -> &str {",
    ));
    let generated_line_terminator = "ifcontains_ecmascript_line_terminator(before_padding)\
        ||contains_ecmascript_line_terminator(after_padding)\
        {returnErr(SpanStableReplacementError::GeneratedLineTerminator);}";
    let generated_width = "letSome(generated_width)=before_padding.len().checked_add(\
        after_padding.len())else{returnErr(SpanStableReplacementError::DoesNotFit);};";
    let required_width = "letSome(required_width)=generated_width.checked_add(terminator_width)\
        .and_then(|width|width.checked_add(internal_barriers))\
        .and_then(|width|width.checked_add(iftrailing_barrier{1}else{0}))\
        else{returnErr(SpanStableReplacementError::DoesNotFit);};";
    let padding = "letSome(padding)=erased.len().checked_sub(required_width)\
        else{returnErr(SpanStableReplacementError::DoesNotFit);};";
    for producer in [
        generated_line_terminator,
        generated_width,
        required_width,
        padding,
    ] {
        assert!(
            around_padding.contains(producer),
            "missing failure producer `{producer}`"
        );
    }
    assert!(
        around_padding.find(generated_line_terminator).unwrap()
            < around_padding.find(generated_width).unwrap()
    );
    assert!(
        around_padding.find(generated_width).unwrap()
            < around_padding.find(required_width).unwrap()
    );
    assert!(around_padding.find(required_width).unwrap() < around_padding.find(padding).unwrap());
    assert_eq!(
        around_padding
            .matches("SpanStableReplacementError::GeneratedLineTerminator")
            .count(),
        1
    );
    assert_eq!(
        around_padding
            .matches("SpanStableReplacementError::DoesNotFit")
            .count(),
        3
    );
}

#[test]
fn default_export_rewrite_exhaustively_maps_each_failure_without_reordering_output() {
    let rewrite = bounded(
        OWNER_SOURCE,
        "fn rewrite_default_keywords(&self, start: usize, end: usize)",
        "fn consume_optional_semicolon(&self, end: usize)",
    );
    assert_eq!(
        code_without_whitespace(bounded(
            rewrite,
            "SourceEdit::replace_around_padding(",
            ".map_err(|error| match error {",
        )),
        "self.source,start,end,&before_padding,DEFAULT_BINDING_ASSIGN,)"
    );
    let route = bounded(rewrite, ".map_err(|error| match error {", "        })");
    assert_eq!(
        code_without_whitespace(route),
        "SpanStableReplacementError::DoesNotFit=>StripError::new(format!(\
         \"`exportdefault`binding`{name}`doesnotfitinthe{width}bytesitreplaces\\\
         afterpreservingitslineterminators\")),\
         SpanStableReplacementError::InvalidSpan=>StripError::new(format!(\
         \"`exportdefault`span{start}..{end}isnotaspanofthismodule'ssourcetext\")),\
         SpanStableReplacementError::GeneratedLineTerminator=>StripError::new(\
         \"generated`exportdefault`declarationheadcontainsalineterminator\",),"
    );
    assert!(!route.contains("_ =>"));

    let focused_witness = bounded(
        OWNER_SOURCE,
        "fn a_generated_replacement_cannot_add_a_line_terminator() {",
        "fn a_hoistable_anonymous_default_is_declared_with_var() {",
    );
    assert_eq!(
        code_without_whitespace(focused_witness),
        "assert!(matches!(SpanStableReplacement::around_padding(\
         \"exportdefault\",\"\",\"let$d0$\\n\",\"=\"),\
         Err(SpanStableReplacementError::GeneratedLineTerminator)));}#[test]"
    );
}
